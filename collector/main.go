package main

import (
	"encoding/json"
	"github.com/gin-gonic/gin"
	"log"
)

func main() {
	r := gin.Default()
	r.POST("/record", func(c *gin.Context) {
		record := &Record{}
		_ = c.BindJSON(record)
		addRecord(record)
		marshal, _ := json.Marshal(record)
		log.Println(string(marshal))
		c.JSON(200, gin.H{
			"status": "OK",
		})
	})
	r.GET("/gen", func(c *gin.Context) {
		stubby := Gen()
		stubby.log()
		c.JSON(200, gin.H{
			"status": "OK",
		})
	})
	r.Run(":80")
}
