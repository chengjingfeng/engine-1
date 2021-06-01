locals {
  portal_config = <<PORTAL
hostName:
clusterIssuer: "letsencrypt-qovery"
ingressClass: "nginx-qovery"
links:
  - name: "Grafana"
    path: "grafana"
    url: "grafana.prometheus.svc.cluster.local"
    port: "80"
  - name: "Prometheus"
    path: "prometheus"
    url: "prometheus-operator-prometheus.prometheus.svc.cluster.local"
    port: "9090"
portal:
  port: "80"
oauthConfig:
  port: "4180"
PORTAL
}

resource "helm_release" "qovery_portal" {
  count = var.test_cluster == "false" ? 0 : 1

  name = "qovery-portal"
  chart = "common/charts/qovery-portal"
  namespace = "prometheus"
  atomic = true
  max_history = 50

  values = [local.portal_config]

  set {
    name= "hostName"
    value = var.portal_hostname
  }

  set {
    name= "portal.title"
    value = "qovery-${var.kubernetes_cluster_name}"
  }

  set {
    name = "externalDnsTarget"
    value = "ac5b4f03c5bf5453490e08d5790fe150-041d435ecb7ebb95.elb.eu-west-3.amazonaws.com"
  }

  set {
    name = "oauthConfig.redirectUrl"
    value ="https://${var.portal_hostname}/oauth2/callback"
  }

  set {
    name = "oauthConfig.upstreams"
    value = "https://${var.portal_hostname}/"
  }

  set {
    name = "oauthConfig.oidcIssuerUrl"
    value = "https://qovery-admin.eu.auth0.com/"
  }

  set {
    name = "oauthConfig.clientId"
    value = "R39yQzfHpZLIU5LqSCRJLmBaUG4Z2jFi"
  }

  set {
    name = "oauthConfig.clientSecret"
    value = "Dq4C7MKFsWHtEscN_gVDRBwvcR1xksroXXZFrVpxpo5a-Ah3vp1SvJdVx7jbE2Lo"
  }

  set {
    name = "oauthConfig.cookieName"
    value = "yummy_cookie"
  }

  set {
    name = "oauthConfig.cookieSecret"
    value = "supersuperqovery"
  }

  set {
    name = "oauthConfig.emailDomains"
    value = "qovery.com"
  }

  depends_on = [
    aws_eks_cluster.eks_cluster,
    helm_release.aws_vpc_cni,
    helm_release.cluster_autoscaler,
    helm_release.nginx_ingress,
    {% if metrics_history_enabled %}
    helm_release.prometheus_operator,
    helm_release.grafana,
    {% endif %}
  ]
}

#resource "null_resource" "set_portal_var" {
#  provisioner "local-exec" {
#    command = <<EOT
#echo "setting externalDNS to Qovery Portal"
#kubectl -n prometheus annotate --overwrite Ingress oauthingress external-dns.alpha.kubernetes.io/target=$(kubectl -n nginx-ingress get svc nginx-ingress-controller -o jsonpath="{.status.loadBalancer.ingress[0].hostname}") || echo "$kind release-name failed"
#EOT
#
#    environment = {
#      KUBECONFIG = local_file.kubeconfig.filename
#      AWS_ACCESS_KEY_ID = "{{ aws_access_key }}"
#      AWS_SECRET_ACCESS_KEY = "{{ aws_secret_key }}"
#      AWS_DEFAULT_REGION = "{{ aws_region }}"
#    }
#  }
#
#  depends_on = [
#    helm_release.qovery_portal
#  ]
#}

