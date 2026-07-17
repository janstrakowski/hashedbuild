import XCTest
import SwiftTreeSitter
import TreeSitterHashbuild

final class TreeSitterHashbuildTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_hashbuild())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Hashbuild grammar")
    }
}
