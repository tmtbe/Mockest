package main

import (
	"github.com/gin-gonic/gin"
	"github.com/go-resty/resty/v2"
	"strconv"
)

func main() {
	index := 0
	r := gin.Default()
	client := resty.New()
	r.GET("/inbound", func(c *gin.Context) {
		postBody := gin.H{
			"message": "this is a post body",
		}
		_, err := client.R().SetHeader("demo", "post").SetBody(postBody).Post("http://outbound-demo/outbound_POST")
		if err != nil {
			c.JSON(400, gin.H{
				"status": "outbound_POST ERROR",
			})
			return
		}
		putBody := gin.H{
			"message": "this is a put body",
		}
		_, err = client.R().SetHeader("demo", "put").SetBody(putBody).Put("http://outbound-demo/outbound_PUT")
		if err != nil {
			c.JSON(400, gin.H{
				"status": "outbound_PUT ERROR",
			})
			return
		}
		for i := 0; i < 3; i++ {
			resp, err := client.R().SetHeader("demo", "get").Get("http://outbound-demo/outbound_GET")
			respIndex := resp.Header().Get("index")
			if respIndex != strconv.Itoa(i) {
				println("need:" + strconv.Itoa(i) + "get:" + respIndex)
				c.JSON(400, gin.H{
					"status": "outbound_GET ERROR",
				})
				return
			}
			if err != nil {
				c.JSON(400, gin.H{
					"status": "outbound_GET ERROR",
				})
				return
			}
		}
		c.JSON(200, gin.H{
			"status": "inbound OK",
		})
	})
	r.POST("/outbound_POST", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"status": "outbound_POST OK",
		})
	})
	r.PUT("/outbound_PUT", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"status": "outbound_PUT OK",
		})
	})
	r.GET("/outbound_GET", func(c *gin.Context) {
		c.Header("index", strconv.Itoa(index))
		c.JSON(200, gin.H{
			"status": "outbound_GET OK",
		})
		index++
	})
	r.Run(":80")
}
