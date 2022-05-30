sudo iptables -t nat -A OUTPUT -p tcp -m owner --uid-owner 1987 -j ACCEPT
sudo iptables -t nat -A OUTPUT -p tcp -j REDIRECT --to-port 15001
./envoy -c envoy.yaml
