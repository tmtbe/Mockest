package main

import (
	"encoding/json"
	"github.com/gin-gonic/gin"
	"log"
)

type Record struct {
	PluginType      string   `json:"plugin_type"`
	TraceID         string   `json:"trace_id"`
	RequestHeaders  []Header `json:"request_headers"`
	RequestBody     string   `json:"request_body"`
	ResponseHeaders []Header `json:"response_headers"`
	ResponseBody    string   `json:"response_body"`
}
type Header []string

func main() {
	r := gin.Default()
	r.POST("/record", func(c *gin.Context) {
		record := &Record{}
		_ = c.BindJSON(record)
		marshal, _ := json.Marshal(record)
		log.Println(string(marshal))
		c.JSON(200, gin.H{
			"status": "OK",
		})
	})
	r.POST("/replay_inbound", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"status": "OK",
		})
	})
	r.POST("/replay_outbound", func(c *gin.Context) {
		record := &Record{}
		data := "{\"plugin_type\":\"outbound_record\",\"trace_id\":\"6943202575227289856\",\"request_headers\":[[\":authority\",\"www.baidu.com\"],[\":path\",\"/\"],[\":method\",\"GET\"],[\":scheme\",\"http\"],[\"user-agent\",\"curl/7.29.0\"],[\"accept\",\"*/*\"],[\"x-forwarded-proto\",\"http\"],[\"x-request-id\",\"6d3e4bc1-2529-4088-8d58-c6ff8eaeb496\"],[\"x-envoy-expected-rq-timeout-ms\",\"15000\"]],\"request_body\":\"\",\"response_headers\":[[\":status\",\"200\"],[\"accept-ranges\",\"bytes\"],[\"cache-control\",\"private, no-cache, no-store, proxy-revalidate, no-transform\"],[\"content-length\",\"2381\"],[\"content-type\",\"text/html\"],[\"date\",\"Thu, 16 Jun 2022 14:08:12 GMT\"],[\"etag\",\"\\\"588604c8-94d\\\"\"],[\"last-modified\",\"Mon, 23 Jan 2017 13:27:36 GMT\"],[\"pragma\",\"no-cache\"],[\"server\",\"envoy\"],[\"set-cookie\",\"BDORZ=27315; max-age=86400; domain=.baidu.com; path=/\"],[\"x-envoy-upstream-service-time\",\"48\"]],\"response_body\":\"PCFET0NUWVBFIGh0bWw+DQo8IS0tU1RBVFVTIE9LLS0+PGh0bWw+IDxoZWFkPjxtZXRhIGh0dHAtZXF1aXY9Y29udGVudC10eXBlIGNvbnRlbnQ9dGV4dC9odG1sO2NoYXJzZXQ9dXRmLTg+PG1ldGEgaHR0cC1lcXVpdj1YLVVBLUNvbXBhdGlibGUgY29udGVudD1JRT1FZGdlPjxtZXRhIGNvbnRlbnQ9YWx3YXlzIG5hbWU9cmVmZXJyZXI+PGxpbmsgcmVsPXN0eWxlc2hlZXQgdHlwZT10ZXh0L2NzcyBocmVmPWh0dHA6Ly9zMS5iZHN0YXRpYy5jb20vci93d3cvY2FjaGUvYmRvcnovYmFpZHUubWluLmNzcz48dGl0bGU+55m+5bqm5LiA5LiL77yM5L2g5bCx55+l6YGTPC90aXRsZT48L2hlYWQ+IDxib2R5IGxpbms9IzAwMDBjYz4gPGRpdiBpZD13cmFwcGVyPiA8ZGl2IGlkPWhlYWQ+IDxkaXYgY2xhc3M9aGVhZF93cmFwcGVyPiA8ZGl2IGNsYXNzPXNfZm9ybT4gPGRpdiBjbGFzcz1zX2Zvcm1fd3JhcHBlcj4gPGRpdiBpZD1sZz4gPGltZyBoaWRlZm9jdXM9dHJ1ZSBzcmM9Ly93d3cuYmFpZHUuY29tL2ltZy9iZF9sb2dvMS5wbmcgd2lkdGg9MjcwIGhlaWdodD0xMjk+IDwvZGl2PiA8Zm9ybSBpZD1mb3JtIG5hbWU9ZiBhY3Rpb249Ly93d3cuYmFpZHUuY29tL3MgY2xhc3M9Zm0+IDxpbnB1dCB0eXBlPWhpZGRlbiBuYW1lPWJkb3J6X2NvbWUgdmFsdWU9MT4gPGlucHV0IHR5cGU9aGlkZGVuIG5hbWU9aWUgdmFsdWU9dXRmLTg+IDxpbnB1dCB0eXBlPWhpZGRlbiBuYW1lPWYgdmFsdWU9OD4gPGlucHV0IHR5cGU9aGlkZGVuIG5hbWU9cnN2X2JwIHZhbHVlPTE+IDxpbnB1dCB0eXBlPWhpZGRlbiBuYW1lPXJzdl9pZHggdmFsdWU9MT4gPGlucHV0IHR5cGU9aGlkZGVuIG5hbWU9dG4gdmFsdWU9YmFpZHU+PHNwYW4gY2xhc3M9ImJnIHNfaXB0X3dyIj48aW5wdXQgaWQ9a3cgbmFtZT13ZCBjbGFzcz1zX2lwdCB2YWx1ZSBtYXhsZW5ndGg9MjU1IGF1dG9jb21wbGV0ZT1vZmYgYXV0b2ZvY3VzPjwvc3Bhbj48c3BhbiBjbGFzcz0iYmcgc19idG5fd3IiPjxpbnB1dCB0eXBlPXN1Ym1pdCBpZD1zdSB2YWx1ZT3nmb7luqbkuIDkuIsgY2xhc3M9ImJnIHNfYnRuIj48L3NwYW4+IDwvZm9ybT4gPC9kaXY+IDwvZGl2PiA8ZGl2IGlkPXUxPiA8YSBocmVmPWh0dHA6Ly9uZXdzLmJhaWR1LmNvbSBuYW1lPXRqX3RybmV3cyBjbGFzcz1tbmF2PuaWsOmXuzwvYT4gPGEgaHJlZj1odHRwOi8vd3d3LmhhbzEyMy5jb20gbmFtZT10al90cmhhbzEyMyBjbGFzcz1tbmF2PmhhbzEyMzwvYT4gPGEgaHJlZj1odHRwOi8vbWFwLmJhaWR1LmNvbSBuYW1lPXRqX3RybWFwIGNsYXNzPW1uYXY+5Zyw5Zu+PC9hPiA8YSBocmVmPWh0dHA6Ly92LmJhaWR1LmNvbSBuYW1lPXRqX3RydmlkZW8gY2xhc3M9bW5hdj7op4bpopE8L2E+IDxhIGhyZWY9aHR0cDovL3RpZWJhLmJhaWR1LmNvbSBuYW1lPXRqX3RydGllYmEgY2xhc3M9bW5hdj7otLTlkKc8L2E+IDxub3NjcmlwdD4gPGEgaHJlZj1odHRwOi8vd3d3LmJhaWR1LmNvbS9iZG9yei9sb2dpbi5naWY/bG9naW4mYW1wO3RwbD1tbiZhbXA7dT1odHRwJTNBJTJGJTJGd3d3LmJhaWR1LmNvbSUyZiUzZmJkb3J6X2NvbWUlM2QxIG5hbWU9dGpfbG9naW4gY2xhc3M9bGI+55m75b2VPC9hPiA8L25vc2NyaXB0PiA8c2NyaXB0PmRvY3VtZW50LndyaXRlKCc8YSBocmVmPSJodHRwOi8vd3d3LmJhaWR1LmNvbS9iZG9yei9sb2dpbi5naWY/bG9naW4mdHBsPW1uJnU9JysgZW5jb2RlVVJJQ29tcG9uZW50KHdpbmRvdy5sb2NhdGlvbi5ocmVmKyAod2luZG93LmxvY2F0aW9uLnNlYXJjaCA9PT0gIiIgPyAiPyIgOiAiJiIpKyAiYmRvcnpfY29tZT0xIikrICciIG5hbWU9InRqX2xvZ2luIiBjbGFzcz0ibGIiPueZu+W9lTwvYT4nKTs8L3NjcmlwdD4gPGEgaHJlZj0vL3d3dy5iYWlkdS5jb20vbW9yZS8gbmFtZT10al9icmlpY29uIGNsYXNzPWJyaSBzdHlsZT0iZGlzcGxheTogYmxvY2s7Ij7mm7TlpJrkuqflk4E8L2E+IDwvZGl2PiA8L2Rpdj4gPC9kaXY+IDxkaXYgaWQ9ZnRDb24+IDxkaXYgaWQ9ZnRDb253PiA8cCBpZD1saD4gPGEgaHJlZj1odHRwOi8vaG9tZS5iYWlkdS5jb20+5YWz5LqO55m+5bqmPC9hPiA8YSBocmVmPWh0dHA6Ly9pci5iYWlkdS5jb20+QWJvdXQgQmFpZHU8L2E+IDwvcD4gPHAgaWQ9Y3A+JmNvcHk7MjAxNyZuYnNwO0JhaWR1Jm5ic3A7PGEgaHJlZj1odHRwOi8vd3d3LmJhaWR1LmNvbS9kdXR5Lz7kvb/nlKjnmb7luqbliY3lv4Xor7s8L2E+Jm5ic3A7IDxhIGhyZWY9aHR0cDovL2ppYW55aS5iYWlkdS5jb20vIGNsYXNzPWNwLWZlZWRiYWNrPuaEj+ingeWPjemmiDwvYT4mbmJzcDvkuqxJQ1Dor4EwMzAxNzPlj7cmbmJzcDsgPGltZyBzcmM9Ly93d3cuYmFpZHUuY29tL2ltZy9ncy5naWY+IDwvcD4gPC9kaXY+IDwvZGl2PiA8L2Rpdj4gPC9ib2R5PiA8L2h0bWw+DQo=\"}"
		_ = json.Unmarshal([]byte(data), &record)
		c.JSON(200, record)
	})
	r.Run(":80")
}
