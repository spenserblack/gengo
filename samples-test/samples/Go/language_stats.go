package main

import (
	"fmt"
	"sort"
)

// Language represents a programming language
type Language struct {
	// Name of the language
	Name string
	// Color associated with the language
	Color string
	// Rank of the language in popularity
	Rank int
	// Year the language was created
	YearCreated int
	// Features of the language
	Features []string
}

// LanguageStats represents a collection of programming languages
type LanguageStats struct {
	// List of languages
	languages []Language
}

// AddLanguage adds a new language to the collection
func (ls *LanguageStats) AddLanguage(lang Language) {
	ls.languages = append(ls.languages, lang)
}

// SortByRank sorts the languages by rank
func (ls *LanguageStats) SortByRank() {
	sort.Slice(ls.languages, func(i, j int) bool {
		return ls.languages[i].Rank < ls.languages[j].Rank
	})
}

func main() {
	stats := LanguageStats{}

	// Add Go
	stats.AddLanguage(Language{
		Name:        "Go",
		Color:       "Blue",
		Rank:        8,
		YearCreated: 2009,
		Features:    []string{"Concurrent", "Compiled", "Static typing"},
	})

	// Add Python
	stats.AddLanguage(Language{
		Name:        "Python",
		Color:       "Yellow and Blue",
		Rank:        3,
		YearCreated: 1991,
		Features:    []string{"Dynamic typing", "Interpreted", "Object-oriented"},
	})

	// Add JavaScript
	stats.AddLanguage(Language{
		Name:        "JavaScript",
		Color:       "Yellow",
		Rank:        1,
		YearCreated: 1995,
		Features:    []string{"Dynamic typing", "Interpreted", "Prototype-based"},
	})

	stats.SortByRank()

	fmt.Println("Programming Languages by Popularity Rank:")
	fmt.Println("----------------------------------------")

	for _, lang := range stats.languages {
		fmt.Printf("%s (Rank: %d)\n", lang.Name, lang.Rank)
		fmt.Printf("Created in: %d\n", lang.YearCreated)
		fmt.Printf("Features: %v\n", lang.Features)
		fmt.Println("----------------------------------------")
	}
}
