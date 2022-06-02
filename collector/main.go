package main

import "github.com/gin-gonic/gin"

func main() {
	r := gin.Default()
	r.POST("/collect", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"message": "pong",
		})
	})
	r.POST("/get_response", func(c *gin.Context) {
		c.JSON(200, gin.H{})
	})
	r.Run(":80")
}
