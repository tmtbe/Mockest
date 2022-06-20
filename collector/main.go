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
	r.Run(":80")
}
