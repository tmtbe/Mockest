package main

import (
	"strings"
)

type Record struct {
	PluginType      string  `json:"plugin_type"`
	TraceID         string  `json:"trace_id"`
	RequestHeaders  Headers `json:"request_headers"`
	RequestBody     string  `json:"request_body"`
	ResponseHeaders Headers `json:"response_headers"`
	ResponseBody    string  `json:"response_body"`
	Index           int     `json:"index"`
}

type Headers []Header

func (h Headers) GetAllHeaders() map[string]string {
	headerMap := make(map[string]string)
	for _, header := range h {
		headerMap[strings.ToLower(header[0])] = header[1]
	}
	return headerMap
}
func (h Headers) GetRespHeaders() map[string]string {
	headerMap := make(map[string]string)
	for _, header := range h {
		if strings.HasPrefix(header[0], ":") || strings.HasPrefix(header[0], "x-") {
			continue
		}
		if header[0] == "user-agent" || header[0] == "content-length" {
			continue
		}
		headerMap[header[0]] = header[1]
	}
	return headerMap
}
func (h Headers) GetReqHeaders() map[string]string {
	headerMap := make(map[string]string)
	for _, header := range h {
		if strings.HasPrefix(header[0], ":") || strings.HasPrefix(header[0], "x-") {
			continue
		}
		if header[0] == "user-agent" || header[0] == "content-length" {
			continue
		}
		headerMap[header[0]] = header[1]
	}
	return headerMap
}

func (h *Headers) GetHeader(name string) string {
	v, ok := h.GetAllHeaders()[name]
	if !ok {
		v = ""
	}
	return v
}

type Header []string
type Records []*Record

var recordMap = make(map[string]Records)

func addRecord(record *Record) {
	traceId := record.TraceID
	if traceId == "" {
		traceId = "untracked"
	}
	if _, ok := recordMap[traceId]; !ok {
		recordMap[record.TraceID] = make([]*Record, 0)
	}
	recordMap[record.TraceID] = append(recordMap[record.TraceID], record)
}

func GetTraceRecord(traceId string) (inbound *Record, outbounds Records) {
	records, ok := recordMap[traceId]
	if !ok {
		return nil, nil
	}
	outbounds = make([]*Record, 0)
	for _, r := range records {
		if r.PluginType == "inbound_record" {
			inbound = r
		} else if r.PluginType == "outbound_record" {
			outbounds = append(outbounds, r)
		}
	}
	return inbound, outbounds
}
