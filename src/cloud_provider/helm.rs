use crate::cloud_provider::helm::HelmAction::Deploy;
use crate::cloud_provider::helm::HelmChartNamespaces::KubeSystem;
use crate::cmd::helm::{helm_exec_uninstall_with_chart_info, helm_exec_upgrade_with_chart_info};
use crate::cmd::kubectl::kubectl_exec_rollout_restart_deployment;
use crate::error::{SimpleError, SimpleErrorKind};
use std::path::Path;
use std::{fs, thread};
use thread::spawn;

#[derive(Clone)]
pub enum HelmAction {
    Deploy,
    Destroy,
    Skip,
}

#[derive(Copy, Clone)]
pub enum HelmChartNamespaces {
    KubeSystem,
    Prometheus,
    Logging,
    CertManager,
    NginxIngress,
    Qovery,
}

#[derive(Clone)]
pub struct ChartSetValue {
    pub key: String,
    pub value: String,
}

#[derive(Clone)]
pub struct ChartInfo {
    pub name: String,
    pub path: String,
    pub namespace: HelmChartNamespaces,
    pub action: HelmAction,
    pub atomic: bool,
    pub force_upgrade: bool,
    pub timeout: String,
    pub dry_run: bool,
    pub wait: bool,
    pub values: Vec<ChartSetValue>,
    pub values_files: Vec<String>,
}

impl Default for ChartInfo {
    fn default() -> ChartInfo {
        ChartInfo {
            name: "undefined".to_string(),
            path: "undefined".to_string(),
            namespace: KubeSystem,
            action: Deploy,
            atomic: true,
            force_upgrade: false,
            timeout: "300s".to_string(),
            dry_run: false,
            wait: true,
            values: Vec::new(),
            values_files: Vec::new(),
        }
    }
}

pub fn get_chart_namespace(namespace: HelmChartNamespaces) -> String {
    match namespace {
        HelmChartNamespaces::KubeSystem => "kube-system",
        HelmChartNamespaces::Prometheus => "prometheus",
        HelmChartNamespaces::Logging => "logging",
        HelmChartNamespaces::CertManager => "cert-manager",
        HelmChartNamespaces::NginxIngress => "nginx-ingress",
        HelmChartNamespaces::Qovery => "qovery",
    }
    .to_string()
}

pub trait HelmChart {
    fn check_prerequisites(&self) -> Result<(), SimpleError> {
        let chart = self.get_chart_info();
        for file in chart.values_files.iter() {
            match fs::metadata(file) {
                Ok(_) => {}
                Err(e) => {
                    return Err(SimpleError {
                        kind: SimpleErrorKind::Other,
                        message: Some(format!(
                            "Can't access helm chart override file {} for chart {}. {:?}",
                            file, chart.name, e
                        )),
                    })
                }
            }
        }
        Ok(())
    }

    fn get_chart_info(&self) -> &ChartInfo;

    fn namespace(&self) -> String {
        get_chart_namespace(self.get_chart_info().namespace)
    }

    fn pre_exec(&self, _kubernetes_config: &Path, _envs: &[(String, String)]) -> Result<(), SimpleError> {
        //
        Ok(())
    }

    fn run(&self, kubernetes_config: &Path, envs: &[(String, String)]) -> Result<(), SimpleError> {
        self.check_prerequisites()?;
        self.pre_exec(&kubernetes_config, &envs)?;
        match self.exec(&kubernetes_config, &envs) {
            Ok(_) => {}
            Err(e) => {
                error!("Error while deploying chart: {:?}", e.message);
                return self.on_deploy_failure(&kubernetes_config, &envs);
            }
        };
        self.post_exec(&kubernetes_config, &envs)?;
        Ok(())
    }

    fn exec(&self, kubernetes_config: &Path, envs: &[(String, String)]) -> Result<(), SimpleError> {
        let environment_variables = envs.iter().map(|x| (x.0.as_str(), x.1.as_str())).collect();
        match self.get_chart_info().action {
            HelmAction::Deploy => {
                helm_exec_upgrade_with_chart_info(kubernetes_config, &environment_variables, self.get_chart_info())
            }
            HelmAction::Destroy => {
                helm_exec_uninstall_with_chart_info(kubernetes_config, &environment_variables, self.get_chart_info())
            }
            HelmAction::Skip => Ok(()),
        }
    }

    fn post_exec(&self, _kubernetes_config: &Path, _envs: &[(String, String)]) -> Result<(), SimpleError> {
        Ok(())
    }
    fn on_deploy_failure(&self, _kubernetes_config: &Path, _envs: &[(String, String)]) -> Result<(), SimpleError> {
        Ok(())
    }
}

//todo: implement it
#[allow(unused_must_use)]
pub fn deploy_multiple_charts<H: 'static + HelmChart + Sync + Send>(
    kubernetes_config: &'static Path,
    envs: &Vec<(String, String)>,
    charts: Vec<H>,
) {
    let mut handles = vec![];

    for chart in charts.into_iter() {
        let environment_variables = envs.clone();
        let kubeconfig = kubernetes_config.clone();
        let handle = spawn(move || chart.run(kubeconfig, &environment_variables));
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

//
// Common charts
//

#[derive(Default)]
pub struct CommonChart {
    pub chart_info: ChartInfo,
}

impl CommonChart {}

impl HelmChart for CommonChart {
    fn get_chart_info(&self) -> &ChartInfo {
        &self.chart_info
    }
}

// CoreDNS config

#[derive(Default)]
pub struct CoreDNSConfigChart {
    pub chart_info: ChartInfo,
}

impl HelmChart for CoreDNSConfigChart {
    fn get_chart_info(&self) -> &ChartInfo {
        &self.chart_info
    }

    // todo: it would be better to avoid rebooting coredns on every run
    fn post_exec(&self, kubernetes_config: &Path, envs: &[(String, String)]) -> Result<(), SimpleError> {
        let environment_variables = envs.iter().map(|x| (x.0.as_str(), x.1.as_str())).collect();

        kubectl_exec_rollout_restart_deployment(
            kubernetes_config,
            &self.chart_info.name,
            self.namespace().as_str(),
            environment_variables,
        )
    }
}
