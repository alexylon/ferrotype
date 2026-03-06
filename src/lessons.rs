pub struct Lesson {
    pub label: &'static str,
    pub text: &'static str,
}

pub const LESSONS: &[Lesson] = &[
    Lesson {
        label: "Pangrams",
        text: "\
The quick brown fox jumps over the lazy dog
Pack my box with five dozen liquor jugs
How vexingly quick daft zebras jump
The five boxing wizards jump quickly
Sphinx of black quartz judge my vow",
    },
    Lesson {
        label: "Home row",
        text: "\
add a sad flask half a salad falls
lads had a glad dash all shall ask
a flash glass hall had a jag slash
fall hadask glad sad lad shall add",
    },
    Lesson {
        label: "Common words",
        text: "\
the quick and simple way to learn is to practice every day
she said that there was nothing left for them after all
they would have been right about most of the other things
some people think it could be done in just a few minutes",
    },
    Lesson {
        label: "Numbers & symbols",
        text: "\
order 47 items at $3.50 each for a total of $164.50
call (555) 123-4567 or email test@example.com today
the ratio is 3:1 and the score was 89% out of 100%
use keys [a-z] and {0-9} to type #hash & @mention",
    },
];
