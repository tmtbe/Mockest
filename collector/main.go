package main

import (
	"github.com/gin-gonic/gin"
	"log"
	"strings"
)

func main() {
	r := gin.Default()
	r.POST("/record", func(c *gin.Context) {
		record := &Record{}
		_ = c.BindJSON(record)
		log.Printf(record.PluginType)
		addRecord(record)
		c.JSON(200, gin.H{
			"status": "OK",
		})
	})
	r.GET("/gen", func(c *gin.Context) {
		stubby := Gen()
		md5Map := stubby.Write("/home")
		msg := ""
		for _, v := range md5Map {
			if len(v) > 1 {
				msg += "[" + strings.Join(v, ",") + "] "
			}
		}
		if msg != "" {
			c.JSON(400, gin.H{
				"status": "Error",
				"msg":    "Duplicate requests, indistinguishable, need to add special header，duplicate trace IDs is: " + msg,
			})
		} else {
			c.JSON(200, gin.H{
				"status": "OK",
			})
		}
	})
	r.Run(":80")
}
