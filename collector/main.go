package main

import "github.com/gin-gonic/gin"

func main() {
	r := gin.Default()
	r.POST("/record", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"message": "pong",
		})
	})
	r.POST("/replay", func(c *gin.Context) {
		c.JSON(200, gin.H{})
	})
	r.Run(":80")
}
