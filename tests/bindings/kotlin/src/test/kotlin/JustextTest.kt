import uniffi.justext_uniffi.*
import kotlin.test.*
import org.junit.jupiter.api.Nested

class JustextTest {

    private val goodParagraph = "This is a sentence that contains many common English stopwords and it " +
        "should be classified as good content by the algorithm because the text is " +
        "long enough that it exceeds the length_high threshold of two hundred characters."

    @Nested inner class ExtractText {

        @Test fun `basic extraction`() {
            val html = "<html><body><p>$goodParagraph</p></body></html>"
            val result = extractText(html, "English")
            assertTrue(result.contains("good content"))
        }

        @Test fun `empty html`() {
            val result = extractText("<html><body></body></html>", "English")
            assertEquals("", result)
        }

        @Test fun `with default config matches extract_text`() {
            val html = "<html><body><p>$goodParagraph</p></body></html>"
            val result = extractTextWith(html, "English", defaultConfig())
            assertEquals(extractText(html, "English"), result)
        }

        @Test fun `unknown language throws`() {
            assertFailsWith<JustextException.UnknownLanguage> {
                extractText("<p>hello</p>", "Klingon")
            }
        }

        @Test fun `filters boilerplate`() {
            val html = """<html><body>
                <p><a>Home</a> | <a>About</a> | <a>Contact</a></p>
                <p>$goodParagraph</p>
            </body></html>"""
            val result = extractText(html, "English")
            assertTrue(result.contains("good content"))
            assertFalse(result.contains("Home"))
        }
    }

    @Nested inner class ClassifyParagraphs {

        @Test fun `good paragraph`() {
            val html = "<html><body><p>$goodParagraph</p></body></html>"
            val paragraphs = classifyParagraphs(html, "English")
            assertTrue(paragraphs.isNotEmpty())
            assertEquals(ClassType.GOOD, paragraphs[0].classType)
        }

        @Test fun `boilerplate`() {
            val html = "<html><body><p><a>Home</a> | <a>About</a> | <a>Contact</a> | <a>Privacy</a> | <a>Terms</a></p></body></html>"
            val paragraphs = classifyParagraphs(html, "English")
            assertTrue(paragraphs.isNotEmpty())
            assertEquals(ClassType.BAD, paragraphs[0].classType)
        }

        @Test fun `empty html`() {
            val paragraphs = classifyParagraphs("<html><body></body></html>", "English")
            assertTrue(paragraphs.isEmpty())
        }

        @Test fun `paragraph fields`() {
            val html = "<html><body><h2>My Heading</h2></body></html>"
            val paragraphs = classifyParagraphs(html, "English")
            assertTrue(paragraphs.isNotEmpty())
            val h = paragraphs[0]
            assertEquals("My Heading", h.text)
            assertTrue(h.domPath.contains("h2"))
            assertTrue(h.xpath.contains("h2"))
            assertTrue(h.heading)
            assertEquals(2.toULong(), h.wordsCount)
        }
    }

    @Nested inner class ConfigAndLanguages {

        @Test fun `default config`() {
            val config = defaultConfig()
            assertEquals(70.toULong(), config.lengthLow)
            assertEquals(200.toULong(), config.lengthHigh)
            assertEquals(0.30, config.stopwordsLow, 0.001)
            assertEquals(0.32, config.stopwordsHigh, 0.001)
            assertEquals(0.2, config.maxLinkDensity, 0.001)
            assertEquals(200.toULong(), config.maxHeadingDistance)
            assertFalse(config.noHeadings)
        }

        @Test fun `available languages`() {
            val langs = availableLanguages()
            assertTrue(langs.size > 50)
            assertTrue(langs.contains("English"))
            assertTrue(langs.contains("French"))
            assertTrue(langs.contains("German"))
        }
    }
}
