package tree_sitter_rlox_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_rlox "github.com/abrishk26/rlox/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_rlox.Language())
	if language == nil {
		t.Errorf("Error loading Rlox grammar")
	}
}
