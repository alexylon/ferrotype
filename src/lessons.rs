use crate::settings::KeyboardLayout;

pub struct Lesson {
    pub id: &'static str,
    pub title: &'static str,
    pub keys: &'static str,
    pub text: &'static str,
}

struct Meta {
    id: &'static str,
    title: &'static str,
}

mod meta {
    use super::Meta;
    pub const INDEX_KEYS: Meta = Meta {
        id: "index_keys",
        title: "Index Keys",
    };
    pub const MIDDLE_KEYS: Meta = Meta {
        id: "middle_keys",
        title: "Middle Keys",
    };
    pub const RING_PINKY: Meta = Meta {
        id: "ring_pinky",
        title: "Ring & Pinky",
    };
    pub const HOME_ROW: Meta = Meta {
        id: "home_row",
        title: "Home Row",
    };
    pub const HOME_REACH: Meta = Meta {
        id: "home_reach",
        title: "Home Reach",
    };
    pub const TOP_INTRO: Meta = Meta {
        id: "top_intro",
        title: "Top Intro",
    };
    pub const TOP_ROW: Meta = Meta {
        id: "top_row",
        title: "Top Row",
    };
    pub const BOTTOM_ROW: Meta = Meta {
        id: "bottom_row",
        title: "Bottom Row",
    };
    pub const ALL_KEYS: Meta = Meta {
        id: "all_keys",
        title: "All Letters",
    };
    pub const CAPITALS: Meta = Meta {
        id: "capitals",
        title: "Capitals & Shift",
    };
    pub const NUMBERS: Meta = Meta {
        id: "numbers",
        title: "0-9 Numbers",
    };
    pub const PUNCTUATION: Meta = Meta {
        id: "punctuation",
        title: "Punctuation & Symbols",
    };
    pub const COMMON_WORDS: Meta = Meta {
        id: "common_words",
        title: "Common Words",
    };
    pub const PARAGRAPHS: Meta = Meta {
        id: "paragraphs",
        title: "Full Paragraphs",
    };
    pub const CODE: Meta = Meta {
        id: "code",
        title: "Code (Rust)",
    };
}

// --- QWERTY layout-specific lessons (1-9) ---

const QWERTY_LESSONS: &[Lesson] = &[
    Lesson {
        id: meta::INDEX_KEYS.id,
        title: meta::INDEX_KEYS.title,
        keys: "f j d k",
        text: include_str!("../assets/lessons/qwerty/01_index_keys.txt"),
    },
    Lesson {
        id: meta::MIDDLE_KEYS.id,
        title: meta::MIDDLE_KEYS.title,
        keys: "d k (+ f j)",
        text: include_str!("../assets/lessons/qwerty/02_middle_keys.txt"),
    },
    Lesson {
        id: meta::RING_PINKY.id,
        title: meta::RING_PINKY.title,
        keys: "s l ; (+ f j d k)",
        text: include_str!("../assets/lessons/qwerty/03_ring_pinky.txt"),
    },
    Lesson {
        id: meta::HOME_ROW.id,
        title: meta::HOME_ROW.title,
        keys: "a s d f j k l ;",
        text: include_str!("../assets/lessons/qwerty/04_home_row.txt"),
    },
    Lesson {
        id: meta::HOME_REACH.id,
        title: meta::HOME_REACH.title,
        keys: "g h",
        text: include_str!("../assets/lessons/qwerty/05_home_reach.txt"),
    },
    Lesson {
        id: meta::TOP_INTRO.id,
        title: meta::TOP_INTRO.title,
        keys: "e i r u",
        text: include_str!("../assets/lessons/qwerty/06_top_intro.txt"),
    },
    Lesson {
        id: meta::TOP_ROW.id,
        title: meta::TOP_ROW.title,
        keys: "q w e r t y u i o p",
        text: include_str!("../assets/lessons/qwerty/07_top_row.txt"),
    },
    Lesson {
        id: meta::BOTTOM_ROW.id,
        title: meta::BOTTOM_ROW.title,
        keys: "z x c v b n m , .",
        text: include_str!("../assets/lessons/qwerty/08_bottom_row.txt"),
    },
    Lesson {
        id: meta::ALL_KEYS.id,
        title: meta::ALL_KEYS.title,
        keys: "",
        text: include_str!("../assets/lessons/qwerty/09_all_keys.txt"),
    },
];

// --- Dvorak layout-specific lessons (1-9) ---

const DVORAK_LESSONS: &[Lesson] = &[
    Lesson {
        id: meta::INDEX_KEYS.id,
        title: meta::INDEX_KEYS.title,
        keys: "u h e t",
        text: include_str!("../assets/lessons/dvorak/01_index_keys.txt"),
    },
    Lesson {
        id: meta::MIDDLE_KEYS.id,
        title: meta::MIDDLE_KEYS.title,
        keys: "e t (+ u h)",
        text: include_str!("../assets/lessons/dvorak/02_middle_keys.txt"),
    },
    Lesson {
        id: meta::RING_PINKY.id,
        title: meta::RING_PINKY.title,
        keys: "o n s (+ a)",
        text: include_str!("../assets/lessons/dvorak/03_ring_pinky.txt"),
    },
    Lesson {
        id: meta::HOME_ROW.id,
        title: meta::HOME_ROW.title,
        keys: "a o e u i d h t n s",
        text: include_str!("../assets/lessons/dvorak/04_home_row.txt"),
    },
    Lesson {
        id: meta::HOME_REACH.id,
        title: meta::HOME_REACH.title,
        keys: "i d",
        text: include_str!("../assets/lessons/dvorak/05_home_reach.txt"),
    },
    Lesson {
        id: meta::TOP_INTRO.id,
        title: meta::TOP_INTRO.title,
        keys: "p c r l",
        text: include_str!("../assets/lessons/dvorak/06_top_intro.txt"),
    },
    Lesson {
        id: meta::TOP_ROW.id,
        title: meta::TOP_ROW.title,
        keys: "' , . p y f g c r l",
        text: include_str!("../assets/lessons/dvorak/07_top_row.txt"),
    },
    Lesson {
        id: meta::BOTTOM_ROW.id,
        title: meta::BOTTOM_ROW.title,
        keys: "; q j k x b m w v z",
        text: include_str!("../assets/lessons/dvorak/08_bottom_row.txt"),
    },
    Lesson {
        id: meta::ALL_KEYS.id,
        title: meta::ALL_KEYS.title,
        keys: "",
        text: include_str!("../assets/lessons/dvorak/09_all_keys.txt"),
    },
];

// --- Colemak layout-specific lessons (1-9) ---

const COLEMAK_LESSONS: &[Lesson] = &[
    Lesson {
        id: meta::INDEX_KEYS.id,
        title: meta::INDEX_KEYS.title,
        keys: "t n s e",
        text: include_str!("../assets/lessons/colemak/01_index_keys.txt"),
    },
    Lesson {
        id: meta::MIDDLE_KEYS.id,
        title: meta::MIDDLE_KEYS.title,
        keys: "s e (+ t n)",
        text: include_str!("../assets/lessons/colemak/02_middle_keys.txt"),
    },
    Lesson {
        id: meta::RING_PINKY.id,
        title: meta::RING_PINKY.title,
        keys: "r i o (+ a)",
        text: include_str!("../assets/lessons/colemak/03_ring_pinky.txt"),
    },
    Lesson {
        id: meta::HOME_ROW.id,
        title: meta::HOME_ROW.title,
        keys: "a r s t d h n e i o",
        text: include_str!("../assets/lessons/colemak/04_home_row.txt"),
    },
    Lesson {
        id: meta::HOME_REACH.id,
        title: meta::HOME_REACH.title,
        keys: "d h",
        text: include_str!("../assets/lessons/colemak/05_home_reach.txt"),
    },
    Lesson {
        id: meta::TOP_INTRO.id,
        title: meta::TOP_INTRO.title,
        keys: "f p l u",
        text: include_str!("../assets/lessons/colemak/06_top_intro.txt"),
    },
    Lesson {
        id: meta::TOP_ROW.id,
        title: meta::TOP_ROW.title,
        keys: "q w f p g j l u y ;",
        text: include_str!("../assets/lessons/colemak/07_top_row.txt"),
    },
    Lesson {
        id: meta::BOTTOM_ROW.id,
        title: meta::BOTTOM_ROW.title,
        keys: "z x c v b k m , .",
        text: include_str!("../assets/lessons/colemak/08_bottom_row.txt"),
    },
    Lesson {
        id: meta::ALL_KEYS.id,
        title: meta::ALL_KEYS.title,
        keys: "",
        text: include_str!("../assets/lessons/colemak/09_all_keys.txt"),
    },
];

// --- Shared lessons (10-15, layout-agnostic) ---

const SHARED_LESSONS: &[Lesson] = &[
    Lesson {
        id: meta::CAPITALS.id,
        title: meta::CAPITALS.title,
        keys: "",
        text: include_str!("../assets/lessons/shared/10_capitals.txt"),
    },
    Lesson {
        id: meta::NUMBERS.id,
        title: meta::NUMBERS.title,
        keys: "",
        text: include_str!("../assets/lessons/shared/11_numbers.txt"),
    },
    Lesson {
        id: meta::PUNCTUATION.id,
        title: meta::PUNCTUATION.title,
        keys: "",
        text: include_str!("../assets/lessons/shared/12_punctuation.txt"),
    },
    Lesson {
        id: meta::COMMON_WORDS.id,
        title: meta::COMMON_WORDS.title,
        keys: "",
        text: include_str!("../assets/lessons/shared/13_common_words.txt"),
    },
    Lesson {
        id: meta::PARAGRAPHS.id,
        title: meta::PARAGRAPHS.title,
        keys: "",
        text: include_str!("../assets/lessons/shared/14_paragraphs.txt"),
    },
    Lesson {
        id: meta::CODE.id,
        title: meta::CODE.title,
        keys: "",
        text: include_str!("../assets/lessons/shared/15_code.txt"),
    },
];

pub fn lessons_for_layout(layout: KeyboardLayout) -> Vec<&'static Lesson> {
    let specific = match layout {
        KeyboardLayout::Qwerty => QWERTY_LESSONS,
        KeyboardLayout::Dvorak => DVORAK_LESSONS,
        KeyboardLayout::Colemak => COLEMAK_LESSONS,
    };
    specific.iter().chain(SHARED_LESSONS.iter()).collect()
}

pub fn lesson_count() -> usize {
    QWERTY_LESSONS.len() + SHARED_LESSONS.len()
}
