// Code generated by jtd-codegen for Go v0.2.1. DO NOT EDIT.

package jtd_codegen_e2e

import (
	"encoding/json"
	"fmt"
)

type Root struct {
	Value IRoot `json:"-"`
}

func (v Root) MarshalJSON() ([]byte, error) {
	return json.Marshal(v.Value)
}

func (v *Root) UnmarshalJSON(b []byte) error {
	var t struct { T string `json:"foo"` }
	if err := json.Unmarshal(b, &t); err != nil {
		return err
	}

	var value IRoot
	var err error

	switch t.T {
	case "BAR_BAZ":
		var v RootBarBaz
		err = json.Unmarshal(b, &v)
		value = v
	case "QUUX":
		var v RootQuux
		err = json.Unmarshal(b, &v)
		value = v
	default:
		err = fmt.Errorf("Root: bad foo value: %q", t.T)
	}

	if err != nil {
		return err
	}

	v.Value = value
	return nil
}

// IRoot is an interface type that Root types implement.
// It can be the following types:
//
// - [RootBarBaz] (BAR_BAZ)
// - [RootQuux] (QUUX)
//
type IRoot interface {
	Foo() string
	isRoot()
}

func (RootBarBaz) Foo() string { return "BAR_BAZ" }
func (RootQuux) Foo() string { return "QUUX" }

func (RootBarBaz) isRoot() {}
func (RootQuux) isRoot() {}

func (v RootBarBaz) MarshalJSON() ([]byte, error) {
	type Alias RootBarBaz
	return json.Marshal(struct {
		T string `json:"foo"`
		Alias
	}{
		v.Foo(),
		Alias(v),
	})
}

func (v *RootBarBaz) UnmarshalJSON(b []byte) error {
	type Alias RootBarBaz
	var a struct {
		T string `json:"foo"`
		Alias
	}

	if err := json.Unmarshal(b, &a); err != nil {
		return err
	}

	if a.T != "BAR_BAZ" {
		return fmt.Errorf("RootBarBaz: bad foo value: %q", a.T)
	}

	*v = RootBarBaz(a.Alias)
	return nil
}

func (v RootQuux) MarshalJSON() ([]byte, error) {
	type Alias RootQuux
	return json.Marshal(struct {
		T string `json:"foo"`
		Alias
	}{
		v.Foo(),
		Alias(v),
	})
}

func (v *RootQuux) UnmarshalJSON(b []byte) error {
	type Alias RootQuux
	var a struct {
		T string `json:"foo"`
		Alias
	}

	if err := json.Unmarshal(b, &a); err != nil {
		return err
	}

	if a.T != "QUUX" {
		return fmt.Errorf("RootQuux: bad foo value: %q", a.T)
	}

	*v = RootQuux(a.Alias)
	return nil
}


type RootBarBaz struct {
	Baz string `json:"baz"`
}

type RootQuux struct {
	Quuz string `json:"quuz"`
}
