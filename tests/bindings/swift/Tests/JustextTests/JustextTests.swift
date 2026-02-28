import XCTest
import Justext

final class JustextTests: XCTestCase {

    private let goodParagraph = """
        This is a sentence that contains many common English stopwords and it \
        should be classified as good content by the algorithm because the text is \
        long enough that it exceeds the length_high threshold of two hundred characters.
        """

    // MARK: - extract_text()

    func testExtractTextBasic() throws {
        let html = "<html><body><p>\(goodParagraph)</p></body></html>"
        let result = try extractText(html: html, language: "English")
        XCTAssertTrue(result.contains("good content"))
    }

    func testExtractTextEmptyHtml() throws {
        let result = try extractText(html: "<html><body></body></html>", language: "English")
        XCTAssertEqual(result, "")
    }

    func testExtractTextWithDefaultConfig() throws {
        let html = "<html><body><p>\(goodParagraph)</p></body></html>"
        let result = try extractTextWith(html: html, language: "English", config: defaultConfig())
        let baseline = try extractText(html: html, language: "English")
        XCTAssertEqual(result, baseline)
    }

    func testExtractTextUnknownLanguage() {
        XCTAssertThrowsError(try extractText(html: "<p>hello</p>", language: "Klingon")) { error in
            guard case JustextError.UnknownLanguage = error else {
                XCTFail("Expected JustextError.UnknownLanguage, got \(error)")
                return
            }
        }
    }

    func testExtractTextFiltersBoilerplate() throws {
        let html = """
            <html><body>
                <p><a>Home</a> | <a>About</a> | <a>Contact</a></p>
                <p>\(goodParagraph)</p>
            </body></html>
            """
        let result = try extractText(html: html, language: "English")
        XCTAssertTrue(result.contains("good content"))
        XCTAssertFalse(result.contains("Home"))
    }

    // MARK: - classify_paragraphs()

    func testClassifyGoodParagraph() throws {
        let html = "<html><body><p>\(goodParagraph)</p></body></html>"
        let paragraphs = try classifyParagraphs(html: html, language: "English")
        XCTAssertFalse(paragraphs.isEmpty)
        XCTAssertEqual(paragraphs[0].classType, ClassType.good)
    }

    func testClassifyBoilerplate() throws {
        let html = "<html><body><p><a>Home</a> | <a>About</a> | <a>Contact</a> | <a>Privacy</a> | <a>Terms</a></p></body></html>"
        let paragraphs = try classifyParagraphs(html: html, language: "English")
        XCTAssertFalse(paragraphs.isEmpty)
        XCTAssertEqual(paragraphs[0].classType, ClassType.bad)
    }

    func testClassifyEmptyHtml() throws {
        let paragraphs = try classifyParagraphs(html: "<html><body></body></html>", language: "English")
        XCTAssertTrue(paragraphs.isEmpty)
    }

    func testParagraphFields() throws {
        let html = "<html><body><h2>My Heading</h2></body></html>"
        let paragraphs = try classifyParagraphs(html: html, language: "English")
        XCTAssertFalse(paragraphs.isEmpty)
        let h = paragraphs[0]
        XCTAssertEqual(h.text, "My Heading")
        XCTAssertTrue(h.domPath.contains("h2"))
        XCTAssertTrue(h.xpath.contains("h2"))
        XCTAssertTrue(h.heading)
        XCTAssertEqual(h.wordCount, 2)
    }

    // MARK: - default config & languages

    func testDefaultConfig() {
        let config = defaultConfig()
        XCTAssertEqual(config.lengthLow, 70)
        XCTAssertEqual(config.lengthHigh, 200)
        XCTAssertEqual(config.stopwordsLow, 0.30, accuracy: 0.001)
        XCTAssertEqual(config.stopwordsHigh, 0.32, accuracy: 0.001)
        XCTAssertEqual(config.maxLinkDensity, 0.2, accuracy: 0.001)
        XCTAssertEqual(config.maxHeadingDistance, 200)
        XCTAssertFalse(config.noHeadings)
    }

    func testAvailableLanguages() {
        let langs = availableLanguages()
        XCTAssertTrue(langs.count > 50)
        XCTAssertTrue(langs.contains("English"))
        XCTAssertTrue(langs.contains("French"))
        XCTAssertTrue(langs.contains("German"))
    }
}
