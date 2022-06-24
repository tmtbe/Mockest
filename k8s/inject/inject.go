package main

import v1 "k8s.io/api/apps/v1"
import cv1 "k8s.io/api/core/v1"

func inject(deployment *v1.Deployment) *v1.Deployment {
	var (
		user                        int64 = 0
		group                       int64 = 0
		proxyUser                   int64 = 1987
		proxyGroup                  int64 = 1987
		runAsNonRoot                      = false
		proxyRunAsNonRoot                 = true
		readOnlyRootFilesystem            = false
		proxyReadOnlyRootFilesystem       = true
		allowPrivilegeEscalation          = false
		privileged                        = false
	)

	initContainer := cv1.Container{
		Name:            "mockest-init",
		Image:           "mockest/proxy:latest",
		Args:            []string{"init"},
		ImagePullPolicy: cv1.PullIfNotPresent,
		SecurityContext: &cv1.SecurityContext{
			Capabilities: &cv1.Capabilities{
				Add: []cv1.Capability{
					"NET_ADMIN",
					"NET_RAW",
				},
				Drop: []cv1.Capability{
					"ALL",
				},
			},
			RunAsUser:                &user,
			RunAsGroup:               &group,
			RunAsNonRoot:             &runAsNonRoot,
			ReadOnlyRootFilesystem:   &readOnlyRootFilesystem,
			AllowPrivilegeEscalation: &allowPrivilegeEscalation,
			Privileged:               &privileged,
		},
	}
	if deployment.Spec.Template.Spec.InitContainers == nil {
		deployment.Spec.Template.Spec.InitContainers = make([]cv1.Container, 0)
	}
	deployment.Spec.Template.Spec.InitContainers = append(deployment.Spec.Template.Spec.InitContainers, initContainer)
	proxyContainer := cv1.Container{
		Name:  "mockest-proxy",
		Image: "mockest/proxy:latest",
		Args:  []string{"proxy"},
		SecurityContext: &cv1.SecurityContext{
			Capabilities: &cv1.Capabilities{
				Drop: []cv1.Capability{
					"ALL",
				},
			},
			RunAsUser:                &proxyUser,
			RunAsGroup:               &proxyGroup,
			RunAsNonRoot:             &proxyRunAsNonRoot,
			ReadOnlyRootFilesystem:   &proxyReadOnlyRootFilesystem,
			AllowPrivilegeEscalation: &allowPrivilegeEscalation,
			Privileged:               &privileged,
		},
	}
	containers := make([]cv1.Container, len(deployment.Spec.Template.Spec.Containers)+1)
	containers[0] = proxyContainer
	for i, c := range deployment.Spec.Template.Spec.Containers {
		containers[i+1] = c
	}
	deployment.Spec.Template.Spec.Containers = containers
	return deployment
}
