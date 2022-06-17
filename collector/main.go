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
		log.Println("replay_inbound")
		c.JSON(200, gin.H{
			"status": "OK",
		})
	})
	r.POST("/replay_outbound", func(c *gin.Context) {
		log.Println("replay_outbound")
		record := &Record{}
		data_tls := "{\"plugin_type\":\"outbound_record\",\"trace_id\":\"6943552262035734784\",\"request_headers\":[[\":authority\",\"www.baidu.com\"],[\":path\",\"/\"],[\":method\",\"GET\"],[\":scheme\",\"https\"],[\"user-agent\",\"curl/7.29.0\"],[\"accept\",\"*/*\"],[\"x-forwarded-proto\",\"https\"],[\"x-request-id\",\"74dafd46-61f5-4018-8b88-09a02a00778f\"],[\"x-envoy-expected-rq-timeout-ms\",\"15000\"]],\"request_body\":\"\",\"response_headers\":[[\":status\",\"200\"],[\"accept-ranges\",\"bytes\"],[\"cache-control\",\"private, no-cache, no-store, proxy-revalidate, no-transform\"],[\"content-length\",\"2443\"],[\"content-type\",\"text/html\"],[\"date\",\"Fri, 17 Jun 2022 13:17:44 GMT\"],[\"etag\",\"\\\"588603eb-98b\\\"\"],[\"last-modified\",\"Mon, 23 Jan 2017 13:23:55 GMT\"],[\"pragma\",\"no-cache\"],[\"server\",\"envoy\"],[\"set-cookie\",\"BDORZ=27315; max-age=86400; domain=.baidu.com; path=/\"],[\"x-envoy-upstream-service-time\",\"84\"]],\"response_body\":\"PCFET0NUWVBFIGh0bWw+DQo8IS0tU1RBVFVTIE9LLS0+PGh0bWw+IDxoZWFkPjxtZXRhIGh0dHAtZXF1aXY9Y29udGVudC10eXBlIGNvbnRlbnQ9dGV4dC9odG1sO2NoYXJzZXQ9dXRmLTg+PG1ldGEgaHR0cC1lcXVpdj1YLVVBLUNvbXBhdGlibGUgY29udGVudD1JRT1FZGdlPjxtZXRhIGNvbnRlbnQ9YWx3YXlzIG5hbWU9cmVmZXJyZXI+PGxpbmsgcmVsPXN0eWxlc2hlZXQgdHlwZT10ZXh0L2NzcyBocmVmPWh0dHBzOi8vc3MxLmJkc3RhdGljLmNvbS81ZU4xYmpxOEFBVVltMnpnb1kzSy9yL3d3dy9jYWNoZS9iZG9yei9iYWlkdS5taW4uY3NzPjx0aXRsZT7nmb7luqbkuIDkuIvvvIzkvaDlsLHnn6XpgZM8L3RpdGxlPjwvaGVhZD4gPGJvZHkgbGluaz0jMDAwMGNjPiA8ZGl2IGlkPXdyYXBwZXI+IDxkaXYgaWQ9aGVhZD4gPGRpdiBjbGFzcz1oZWFkX3dyYXBwZXI+IDxkaXYgY2xhc3M9c19mb3JtPiA8ZGl2IGNsYXNzPXNfZm9ybV93cmFwcGVyPiA8ZGl2IGlkPWxnPiA8aW1nIGhpZGVmb2N1cz10cnVlIHNyYz0vL3d3dy5iYWlkdS5jb20vaW1nL2JkX2xvZ28xLnBuZyB3aWR0aD0yNzAgaGVpZ2h0PTEyOT4gPC9kaXY+IDxmb3JtIGlkPWZvcm0gbmFtZT1mIGFjdGlvbj0vL3d3dy5iYWlkdS5jb20vcyBjbGFzcz1mbT4gPGlucHV0IHR5cGU9aGlkZGVuIG5hbWU9YmRvcnpfY29tZSB2YWx1ZT0xPiA8aW5wdXQgdHlwZT1oaWRkZW4gbmFtZT1pZSB2YWx1ZT11dGYtOD4gPGlucHV0IHR5cGU9aGlkZGVuIG5hbWU9ZiB2YWx1ZT04PiA8aW5wdXQgdHlwZT1oaWRkZW4gbmFtZT1yc3ZfYnAgdmFsdWU9MT4gPGlucHV0IHR5cGU9aGlkZGVuIG5hbWU9cnN2X2lkeCB2YWx1ZT0xPiA8aW5wdXQgdHlwZT1oaWRkZW4gbmFtZT10biB2YWx1ZT1iYWlkdT48c3BhbiBjbGFzcz0iYmcgc19pcHRfd3IiPjxpbnB1dCBpZD1rdyBuYW1lPXdkIGNsYXNzPXNfaXB0IHZhbHVlIG1heGxlbmd0aD0yNTUgYXV0b2NvbXBsZXRlPW9mZiBhdXRvZm9jdXM9YXV0b2ZvY3VzPjwvc3Bhbj48c3BhbiBjbGFzcz0iYmcgc19idG5fd3IiPjxpbnB1dCB0eXBlPXN1Ym1pdCBpZD1zdSB2YWx1ZT3nmb7luqbkuIDkuIsgY2xhc3M9ImJnIHNfYnRuIiBhdXRvZm9jdXM+PC9zcGFuPiA8L2Zvcm0+IDwvZGl2PiA8L2Rpdj4gPGRpdiBpZD11MT4gPGEgaHJlZj1odHRwOi8vbmV3cy5iYWlkdS5jb20gbmFtZT10al90cm5ld3MgY2xhc3M9bW5hdj7mlrDpl7s8L2E+IDxhIGhyZWY9aHR0cHM6Ly93d3cuaGFvMTIzLmNvbSBuYW1lPXRqX3RyaGFvMTIzIGNsYXNzPW1uYXY+aGFvMTIzPC9hPiA8YSBocmVmPWh0dHA6Ly9tYXAuYmFpZHUuY29tIG5hbWU9dGpfdHJtYXAgY2xhc3M9bW5hdj7lnLDlm748L2E+IDxhIGhyZWY9aHR0cDovL3YuYmFpZHUuY29tIG5hbWU9dGpfdHJ2aWRlbyBjbGFzcz1tbmF2PuinhumikTwvYT4gPGEgaHJlZj1odHRwOi8vdGllYmEuYmFpZHUuY29tIG5hbWU9dGpfdHJ0aWViYSBjbGFzcz1tbmF2Pui0tOWQpzwvYT4gPG5vc2NyaXB0PiA8YSBocmVmPWh0dHA6Ly93d3cuYmFpZHUuY29tL2Jkb3J6L2xvZ2luLmdpZj9sb2dpbiZhbXA7dHBsPW1uJmFtcDt1PWh0dHAlM0ElMkYlMkZ3d3cuYmFpZHUuY29tJTJmJTNmYmRvcnpfY29tZSUzZDEgbmFtZT10al9sb2dpbiBjbGFzcz1sYj7nmbvlvZU8L2E+IDwvbm9zY3JpcHQ+IDxzY3JpcHQ+ZG9jdW1lbnQud3JpdGUoJzxhIGhyZWY9Imh0dHA6Ly93d3cuYmFpZHUuY29tL2Jkb3J6L2xvZ2luLmdpZj9sb2dpbiZ0cGw9bW4mdT0nKyBlbmNvZGVVUklDb21wb25lbnQod2luZG93LmxvY2F0aW9uLmhyZWYrICh3aW5kb3cubG9jYXRpb24uc2VhcmNoID09PSAiIiA/ICI/IiA6ICImIikrICJiZG9yel9jb21lPTEiKSsgJyIgbmFtZT0idGpfbG9naW4iIGNsYXNzPSJsYiI+55m75b2VPC9hPicpOw0KICAgICAgICAgICAgICAgIDwvc2NyaXB0PiA8YSBocmVmPS8vd3d3LmJhaWR1LmNvbS9tb3JlLyBuYW1lPXRqX2JyaWljb24gY2xhc3M9YnJpIHN0eWxlPSJkaXNwbGF5OiBibG9jazsiPuabtOWkmuS6p+WTgTwvYT4gPC9kaXY+IDwvZGl2PiA8L2Rpdj4gPGRpdiBpZD1mdENvbj4gPGRpdiBpZD1mdENvbnc+IDxwIGlkPWxoPiA8YSBocmVmPWh0dHA6Ly9ob21lLmJhaWR1LmNvbT7lhbPkuo7nmb7luqY8L2E+IDxhIGhyZWY9aHR0cDovL2lyLmJhaWR1LmNvbT5BYm91dCBCYWlkdTwvYT4gPC9wPiA8cCBpZD1jcD4mY29weTsyMDE3Jm5ic3A7QmFpZHUmbmJzcDs8YSBocmVmPWh0dHA6Ly93d3cuYmFpZHUuY29tL2R1dHkvPuS9v+eUqOeZvuW6puWJjeW/heivuzwvYT4mbmJzcDsgPGEgaHJlZj1odHRwOi8vamlhbnlpLmJhaWR1LmNvbS8gY2xhc3M9Y3AtZmVlZGJhY2s+5oSP6KeB5Y+N6aaIPC9hPiZuYnNwO+S6rElDUOivgTAzMDE3M+WPtyZuYnNwOyA8aW1nIHNyYz0vL3d3dy5iYWlkdS5jb20vaW1nL2dzLmdpZj4gPC9wPiA8L2Rpdj4gPC9kaXY+IDwvZGl2PiA8L2JvZHk+IDwvaHRtbD4NCg==\"}"
		_ = json.Unmarshal([]byte(data_tls), &record)
		c.JSON(200, record)
	})
	r.Run(":80")
}
