#!/usr/bin/env bash
# What JUnit 3 (junit.framework) and JUnit 4 (@Test) pick as a test in a class.
# JUnit itself picks no file: JUnitCore runs the classes that it is given.
# usage: nix-shell -p jdk --run "bash run.sh <dir with junit-4.13.2.jar and hamcrest-core-1.3.jar> <empty dir>"
set -u
JARS=$1
D=$2
cd "$D" || exit 1
CP="$JARS/junit-4.13.2.jar:$JARS/hamcrest-core-1.3.jar"
cat >ShopCase.java <<'EOF'
import junit.framework.TestCase;

public class ShopCase extends TestCase {        // JUnit 3: a TestCase subclass
    public void testLend() { assertEquals(1, make()); }
    public void testWithInput(int n) { }        // has a parameter: not a test
    void testNotPublic() { }                    // not public: a warning test
    public int make() { return 1; }             // a public helper, no "test"
}
EOF
cat >Lending.java <<'EOF'
import org.junit.Test;
import static org.junit.Assert.assertEquals;

public class Lending {                          // JUnit 4: any class name
    @Test public void lends() { assertEquals(1, make()); }
    public void testLooksLikeOne() { }          // "test" name, no @Test: not a test
    public int make() { return 1; }
}
EOF
echo "== java -version"
java -version 2>&1 | head -1
javac -cp "$CP" ShopCase.java Lending.java
echo "== JUnitCore ShopCase (JUnit 3 style)"
java -cp "$CP:." org.junit.runner.JUnitCore ShopCase 2>&1 | grep -E '^OK|^Tests run|Failures|warning|isn.t public|^[0-9]+\)'
echo "== JUnitCore Lending (JUnit 4 style)"
java -cp "$CP:." org.junit.runner.JUnitCore Lending 2>&1 | grep -E '^OK|^Tests run'
