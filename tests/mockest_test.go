package tests

import (
	"github.com/stretchr/testify/assert"
	"net/http"
	"testing"
)

func TestReplay(t *testing.T) {
	{
		resp, err := http.Get("http://proxy/inbound")
		if err != nil {
			assert.Nil(t, err)
		}
		assert.Equal(t, 200, resp.StatusCode)
	}
}
