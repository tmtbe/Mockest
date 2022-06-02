package main

import "github.com/gin-gonic/gin"

func main() {
	r := gin.Default()
	r.POST("/collect", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"message": "pong",
		})
	})
	r.Run(":80")
}
