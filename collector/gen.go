package main

import (
	"github.com/google/uuid"
	"gopkg.in/yaml.v2"
	"log"
	"path/filepath"
	"strconv"
	"strings"
)

type Stubby struct {
	StubbyFiles []*StubbyFile
	Includes    []string `json:"includes" yaml:"includes"`
}

func (s Stubby) log() {
	marshal, _ := yaml.Marshal(s)
	log.Println(string(marshal))
	marshal, _ = yaml.Marshal(s.StubbyFiles)
	log.Println(string(marshal))
}

type StubbyFile struct {
	Name      string            `json:"name" yaml:"name"`
	StubbyMod []*StubbyMod      `json:"stubbyMod" yaml:"stubbyMod"`
	Files     map[string]string `json:"files" yaml:"files"`
}

type StubbyMod struct {
	Request  Request  `json:"request" yaml:"request"`
	Response Response `json:"response" yaml:"response"`
}
type Request struct {
	URL     string            `json:"url" yaml:"url"`
	Method  string            `json:"method" yaml:"method"`
	Headers map[string]string `json:"headers,omitempty" yaml:"headers,omitempty"`
	File    *string           `json:"file,omitempty" yaml:"file,omitempty"`
}
type Response struct {
	Headers map[string]string `json:"headers" yaml:"headers"`
	Status  int               `json:"status" yaml:"status"`
	File    *string           `json:"file,omitempty" yaml:"file,omitempty"`
}

func Gen() Stubby {
	var stubby = Stubby{
		StubbyFiles: make([]*StubbyFile, 0),
		Includes:    make([]string, 0),
	}
	for traceId, _ := range recordMap {
		inbound, outbounds := GetTraceRecord(traceId)
		stubbyFile := &StubbyFile{
			Name:      genName(inbound) + "_" + traceId,
			StubbyMod: make([]*StubbyMod, 0),
			Files:     make(map[string]string),
		}
		stubbyFile.StubbyMod = append(stubbyFile.StubbyMod, stubbyFile.genStubby(inbound))
		for _, outbound := range outbounds {
			stubbyFile.StubbyMod = append(stubbyFile.StubbyMod, stubbyFile.genStubby(outbound))
		}
		stubby.StubbyFiles = append(stubby.StubbyFiles, stubbyFile)
		stubby.Includes = append(stubby.Includes, stubbyFile.Name+".yaml")
	}
	return stubby
}

func genName(record *Record) string {
	names := make([]string, 0)
	names = append(names, record.RequestHeaders.GetHeader(":authority"))
	names = append(names, record.RequestHeaders.GetHeader(":method"))
	names = append(names, strings.ReplaceAll(strings.Trim(strings.Split(record.RequestHeaders.GetHeader(":path"), "?")[0], "/"), "/", "-"))
	names = append(names, record.ResponseHeaders.GetHeader(":status"))
	return strings.ToLower(strings.Join(names, "_"))
}
func (s *StubbyFile) genStubby(record *Record) *StubbyMod {
	if record.PluginType == "inbound_record" {
		inboundResponseHeaders := make(map[string]string)
		inboundResponseHeaders["r_match_type"] = "r_match_inbound"
		inboundResponseHeaders["r_inbound_trace_id"] = record.TraceID
		return &StubbyMod{
			Request: Request{
				URL:     record.RequestHeaders.GetHeader(":path"),
				Method:  record.RequestHeaders.GetHeader(":method"),
				Headers: record.RequestHeaders.GetReqHeaders(),
				File:    s.genBodyFile(record.RequestBody, "request", record),
			},
			Response: Response{
				Headers: inboundResponseHeaders,
				Status:  200,
			},
		}
	} else if record.PluginType == "outbound_record" {
		status, _ := strconv.Atoi(record.ResponseHeaders.GetHeader(":status"))
		outboundRequestHeaders := record.RequestHeaders.GetReqHeaders()
		outboundRequestHeaders["r_match_type"] = "r_match_outbound"
		outboundRequestHeaders["r_inbound_trace_id"] = record.TraceID
		outboundRequestHeaders["r_authority"] = record.RequestHeaders.GetHeader(":authority")
		return &StubbyMod{
			Request: Request{
				URL:     record.RequestHeaders.GetHeader(":path"),
				Method:  record.RequestHeaders.GetHeader(":method"),
				Headers: outboundRequestHeaders,
				File:    s.genBodyFile(record.RequestBody, "request", record),
			},
			Response: Response{
				Headers: record.RequestHeaders.GetRespHeaders(),
				Status:  status,
				File:    s.genBodyFile(record.ResponseBody, "response", record),
			},
		}
	} else {
		return nil
	}
}

func (s *StubbyFile) genBodyFile(body string, opName string, record *Record) *string {
	if body == "" {
		return nil
	}
	name := filepath.Join(s.Name+"_data", opName, genName(record), uuid.New().String())
	s.Files[name] = body
	return &name
}
