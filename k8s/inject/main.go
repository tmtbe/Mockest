package main

import (
	"github.com/spf13/cobra"
	v1 "k8s.io/api/apps/v1"
	"os"
	"sigs.k8s.io/yaml"
)

func main() {
	var rootCmd = &cobra.Command{
		Use:   "inject",
		Short: "inject cmd",
		Run:   injectRun,
	}
	rootCmd.Flags().StringP("file", "f", "", "file")
	err := rootCmd.Execute()
	if err != nil {
		panic(err)
	}
}

func injectRun(c *cobra.Command, _ []string) {
	fileName, err := c.Flags().GetString("file")
	if err != nil {
		panic(err)
	}
	k8sFile, err := os.ReadFile(fileName)
	if err != nil {
		panic(err)
	}
	deployment := &v1.Deployment{}
	inject(deployment)
	err = yaml.Unmarshal(k8sFile, deployment)
	if err != nil {
		panic(err)
	}
	marshal, err := yaml.Marshal(deployment)
	if err != nil {
		panic(err)
	}
	println(string(marshal))
}
