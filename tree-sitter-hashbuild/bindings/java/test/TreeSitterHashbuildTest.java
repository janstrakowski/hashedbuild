import io.github.treesitter.jtreesitter.Language;
import io.github.treesitter.jtreesitter.hashbuild.TreeSitterHashbuild;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;

public class TreeSitterHashbuildTest {
    @Test
    public void testCanLoadLanguage() {
        assertDoesNotThrow(() -> new Language(TreeSitterHashbuild.language()));
    }
}
