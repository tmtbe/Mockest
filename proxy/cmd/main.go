package main

import (
	"fmt"
	"github.com/spf13/cobra"
	"os"
	"os/exec"
)

func main() {
	var rootCmd = &cobra.Command{
		Use:   "proxy",
		Short: "proxy cmd",
	}
	var initCmd = &cobra.Command{
		Use:   "init",
		Short: "init container",
		Run:   initRun,
	}
	var proxyCmd = &cobra.Command{
		Use:   "proxy",
		Short: "proxy",
		Run:   proxyRun,
	}
	proxyCmd.Flags().Bool("replay", false, "is replay mod")
	var allCmd = &cobra.Command{
		Use:   "all",
		Short: "init and proxy",
		Run:   allRun,
	}
	allCmd.Flags().Bool("replay", false, "is replay mod")
	rootCmd.AddCommand(initCmd)
	rootCmd.AddCommand(proxyCmd)
	rootCmd.AddCommand(allCmd)
	err := rootCmd.Execute()
	if err != nil {
		panic(err)
	}
}

func initRun(*cobra.Command, []string) {
	setIptablesWithRoot()
}

func proxyRun(c *cobra.Command, _ []string) {
	replay, _ := c.Flags().GetBool("replay")
	runEnvoy(replay)
}

func allRun(c *cobra.Command, _ []string) {
	replay, _ := c.Flags().GetBool("replay")
	setIptables()
	runEnvoy(replay)
}

func runCmd(name string, args ...string) {
	cmd := exec.Command(name, args...)
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout

	err := cmd.Run()
	if err != nil {
		fmt.Println(err)
	}
}

func setIptables() {
	runCmd("sudo", "iptables", "-t", "nat", "-A", "INPUT", "-p", "tcp", "-j", "ACCEPT")
	runCmd("sudo", "iptables", "-t", "nat", "-A", "OUTPUT", "-p", "tcp", "-m", "owner", "--uid-owner", "1987", "-j", "ACCEPT")
	runCmd("sudo", "iptables", "-t", "nat", "-A", "OUTPUT", "-p", "tcp", "-j", "REDIRECT", "--to-ports", "15001")
	runCmd("sudo", "iptables", "-t", "nat", "-A", "PREROUTING", "-p", "tcp", "-j", "REDIRECT", "--to-ports", "15000")
}
func setIptablesWithRoot() {
	runCmd("iptables", "-t", "nat", "-A", "INPUT", "-p", "tcp", "-j", "ACCEPT")
	runCmd("iptables", "-t", "nat", "-A", "OUTPUT", "-p", "tcp", "-m", "owner", "--uid-owner", "1987", "-j", "ACCEPT")
	runCmd("iptables", "-t", "nat", "-A", "OUTPUT", "-p", "tcp", "-j", "REDIRECT", "--to-ports", "15001")
	runCmd("iptables", "-t", "nat", "-A", "PREROUTING", "-p", "tcp", "-j", "REDIRECT", "--to-ports", "15000")
}
func runEnvoy(replay bool) {
	if !replay {
		runCmd("./envoy", "-c", "envoy-record.yaml")
	} else {
		runCmd("./envoy", "-c", "envoy-replay.yaml")
	}
}
