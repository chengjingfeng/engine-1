resource "helm_release" "metrics_server" {
  name = "metrics-server"
  chart = "common/charts/metrics-server"
  namespace = "kube-system"
  atomic = true
  max_history = 50

  set {
    name = "resources.limits.cpu"
    value = "250m"
  }

  set {
    name = "resources.requests.cpu"
    value = "250m"
  }

  set {
    name = "resources.limits.memory"
    value = "256Mi"
  }

  set {
    name = "resources.requests.memory"
    value = "256Mi"
  }

  set {
    name = "forced_upgrade"
    value = var.forced_upgrade
  }

  depends_on = [
    aws_eks_cluster.eks_cluster,
    helm_release.aws_vpc_cni,
    helm_release.cluster_autoscaler,
  ]
}