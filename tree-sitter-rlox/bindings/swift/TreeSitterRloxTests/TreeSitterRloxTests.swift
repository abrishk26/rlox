import XCTest
import SwiftTreeSitter
import TreeSitterRlox

final class TreeSitterRloxTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_rlox())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Rlox grammar")
    }
}
