sudo iptables -t nat -A INPUT -p tcp -j ACCEPT
sudo iptables -t nat -A OUTPUT -p tcp -m owner --uid-owner 1987 -j ACCEPT
sudo iptables -t nat -A OUTPUT -p tcp -j REDIRECT --to-ports 15001
sudo iptables -t nat -A PREROUTING -p tcp -j REDIRECT --to-ports 15000
if [ -z "$REPLAY" ];then
	./envoy -c envoy-record.yaml
else
	./envoy -c envoy-replay.yaml
fi

