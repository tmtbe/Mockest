package main

import (
	"github.com/gin-gonic/gin"
)

func main() {
	r := gin.Default()
	r.POST("/record", func(c *gin.Context) {
		record := &Record{}
		_ = c.BindJSON(record)
		addRecord(record)
		c.JSON(200, gin.H{
			"status": "OK",
		})
	})
	r.GET("/gen", func(c *gin.Context) {
		stubby := Gen()
		stubby.Write("/home")
		c.JSON(200, gin.H{
			"status": "OK",
		})
	})
	r.Run(":80")
}
