//! unicode字符函数，获得字符的语言区间段。及根据文字排版的需要，判断字符是否为单字字符或字母字符
//! http://www.cnblogs.com/chenwenbiao/archive/2011/08/17/2142718.html
//! 以下是unicode中常见语言的区间段
//! 0000	007F	C0控制符及基本拉丁文	C0 Control and Basic Latin
//! 0080	00FF	C1控制符及拉丁文补充-1	C1 Control and Latin 1 Supplement
//! 0100	017F	拉丁文扩展-A	Latin Extended-A
//! 0180	024F	拉丁文扩展-B	Latin Extended-B
//! 0250	02AF	国际音标扩展	IPA Extensions
//! 02B0	02FF	空白修饰字母	Spacing Modifiers
//! 0300	036F	结合用读音符号	Combining Diacritics Marks
//! 0370	03FF	希腊文及科普特文	Greek and Coptic
//! 0400	04FF	西里尔字母	Cyrillic
//! 0500	052F	西里尔字母补充	Cyrillic Supplement
//! 0530	058F	亚美尼亚语	Armenian
//! 0590	05FF	希伯来文	Hebrew
//! 0600	06FF	阿拉伯文	Arabic
//! 0700	074F	叙利亚文	Syriac
//! 0750	077F	阿拉伯文补充	Arabic Supplement
//! 0780	07BF	马尔代夫语	Thaana
//! 07C0	07FF	西非書面語言	N'Ko
//! 0800	085F	阿维斯塔语及巴列维语	Avestan and Pahlavi
//! 0860	087F	Mandaic	Mandaic
//! 0880	08AF	撒马利亚语	Samaritan
//! 0900	097F	天城文书	Devanagari
//! 0980	09FF	孟加拉语	Bengali
//! 0A00	0A7F	锡克教文	Gurmukhi
//! 0A80	0AFF	古吉拉特文	Gujarati
//! 0B00	0B7F	奥里亚文	Oriya
//! 0B80	0BFF	泰米尔文	Tamil
//! 0C00	0C7F	泰卢固文	Telugu
//! 0C80	0CFF	卡纳达文	Kannada
//! 0D00	0D7F	德拉维族语	Malayalam
//! 0D80	0DFF	僧伽罗语	Sinhala
//! 0E00	0E7F	泰文	Thai
//! 0E80	0EFF	老挝文	Lao
//! 0F00	0FFF	藏文	Tibetan
//! 1000	109F	缅甸语	Myanmar
//! 10A0	10FF	格鲁吉亚语	Georgian
//! 1100	11FF	朝鲜文	Hangul Jamo
//! 1200	137F	埃塞俄比亚语	Ethiopic
//! 1380	139F	埃塞俄比亚语补充	Ethiopic Supplement
//! 13A0	13FF	切罗基语	Cherokee
//! 1400	167F	统一加拿大土著语音节	Unified Canadian Aboriginal Syllabics
//! 1680	169F	欧甘字母	Ogham
//! 16A0	16FF	如尼文	Runic
//! 1700	171F	塔加拉语	Tagalog
//! 1720	173F	Hanunóo	Hanunoo
//! 1740	175F	Buhid	Buhid
//! 1760	177F	Tagbanwa	Tagbanwa
//! 1780	17FF	高棉语	Khmer
//! 1800	18AF	蒙古文	Mongolian
//! 18B0	18FF	Cham	Cham
//! 1900	194F	Limbu	Limbu
//! 1950	197F	德宏泰语	Tai Le
//! 1980	19DF	新傣仂语	New Tai Lue
//! 19E0	19FF	高棉语记号	Kmer Symbols
//! 1A00	1A1F	Buginese	Buginese
//! 1A20	1A5F	Batak	Batak
//! 1A80	1AEF	Lanna	Lanna
//! 1B00	1B7F	巴厘语	Balinese
//! 1B80	1BB0	巽他语	Sundanese
//! 1BC0	1BFF	Pahawh Hmong	Pahawh Hmong
//! 1C00	1C4F	雷布查语	Lepcha
//! 1C50	1C7F	Ol Chiki	Ol Chiki
//! 1C80	1CDF	曼尼普尔语	Meithei/Manipuri
//! 1D00	1D7F	语音学扩展	Phonetic Extensions
//! 1D80	1DBF	语音学扩展补充	Phonetic Extensions Supplement
//! 1DC0	1DFF	结合用读音符号补充	Combining Diacritics Marks Supplement
//! 1E00	1EFF	拉丁文扩充附加	Latin Extended Additional
//! 1F00	1FFF	希腊语扩充	Greek Extended
//! 2000	206F	常用标点	General Punctuation
//! 2070	209F	上标及下标	Superscripts and Subscripts
//! 20A0	20CF	货币符号	Currency Symbols
//! 20D0	20FF	组合用记号	Combining Diacritics Marks for Symbols
//! 2100	214F	字母式符号	Letterlike Symbols
//! 2150	218F	数字形式	Number Form
//! 2190	21FF	箭头	Arrows
//! 2200	22FF	数学运算符	Mathematical Operator
//! 2300	23FF	杂项工业符号	Miscellaneous Technical
//! 2400	243F	控制图片	Control Pictures
//! 2440	245F	光学识别符	Optical Character Recognition
//! 2460	24FF	封闭式字母数字	Enclosed Alphanumerics
//! 2500	257F	制表符	Box Drawing
//! 2580	259F	方块元素	Block Element
//! 25A0	25FF	几何图形	Geometric Shapes
//! 2600	26FF	杂项符号	Miscellaneous Symbols
//! 2700	27BF	印刷符号	Dingbats
//! 27C0	27EF	杂项数学符号-A	Miscellaneous Mathematical Symbols-A
//! 27F0	27FF	追加箭头-A	Supplemental Arrows-A
//! 2800	28FF	盲文点字模型	Braille Patterns
//! 2900	297F	追加箭头-B	Supplemental Arrows-B
//! 2980	29FF	杂项数学符号-B	Miscellaneous Mathematical Symbols-B
//! 2A00	2AFF	追加数学运算符	Supplemental Mathematical Operator
//! 2B00	2BFF	杂项符号和箭头	Miscellaneous Symbols and Arrows
//! 2C00	2C5F	格拉哥里字母	Glagolitic
//! 2C60	2C7F	拉丁文扩展-C	Latin Extended-C
//! 2C80	2CFF	古埃及语	Coptic
//! 2D00	2D2F	格鲁吉亚语补充	Georgian Supplement
//! 2D30	2D7F	提非纳文	Tifinagh
//! 2D80	2DDF	埃塞俄比亚语扩展	Ethiopic Extended
//! 2E00	2E7F	追加标点	Supplemental Punctuation
//! 2E80	2EFF	CJK 部首补充	CJK Radicals Supplement
//! 2F00	2FDF	康熙字典部首	Kangxi Radicals
//! 2FF0	2FFF	表意文字描述符	Ideographic Description Characters
//! 3000	303F	CJK 符号和标点	CJK Symbols and Punctuation
//! 3040	309F	日文平假名	Hiragana
//! 30A0	30FF	日文片假名	Katakana
//! 3100	312F	注音字母	Bopomofo
//! 3130	318F	朝鲜文兼容字母	Hangul Compatibility Jamo
//! 3190	319F	象形字注释标志	Kanbun
//! 31A0	31BF	注音字母扩展	Bopomofo Extended
//! 31C0	31EF	CJK 笔画	CJK Strokes
//! 31F0	31FF	日文片假名语音扩展	Katakana Phonetic Extensions
//! 3200	32FF	封闭式 CJK 文字和月份	Enclosed CJK Letters and Months
//! 3300	33FF	CJK 兼容	CJK Compatibility
//! 3400	4DBF	CJK 统一表意符号扩展 A	CJK Unified Ideographs Extension A
//! 4DC0	4DFF	易经六十四卦符号	Yijing Hexagrams Symbols
//! 4E00	9FBF	CJK 统一表意符号	CJK Unified Ideographs
//! A000	A48F	彝文音节	Yi Syllables
//! A490	A4CF	彝文字根	Yi Radicals
//! A500	A61F	Vai	Vai
//! A660	A6FF	统一加拿大土著语音节补充	Unified Canadian Aboriginal Syllabics Supplement
//! A700	A71F	声调修饰字母	Modifier Tone Letters
//! A720	A7FF	拉丁文扩展-D	Latin Extended-D
//! A800	A82F	Syloti Nagri	Syloti Nagri
//! A840	A87F	八思巴字	Phags-pa
//! A880	A8DF	Saurashtra	Saurashtra
//! A900	A97F	爪哇语	Javanese
//! A980	A9DF	Chakma	Chakma
//! AA00	AA3F	Varang Kshiti	Varang Kshiti
//! AA40	AA6F	Sorang Sompeng	Sorang Sompeng
//! AA80	AADF	Newari	Newari
//! AB00	AB5F	越南傣语	Vit Thai
//! AB80	ABA0	Kayah Li	Kayah Li
//! AC00	D7AF	朝鲜文音节	Hangul Syllables
//! D800	DBFF	High-half zone of UTF-16	High-half zone of UTF-16
//! DC00	DFFF	Low-half zone of UTF-16	Low-half zone of UTF-16
//! E000	F8FF	自行使用区域	Private Use Zone
//! F900	FAFF	CJK 兼容象形文字	CJK Compatibility Ideographs
//! FB00	FB4F	字母表达形式	Alphabetic Presentation Form
//! FB50	FDFF	阿拉伯表达形式A	Arabic Presentation Form-A
//! FE00	FE0F	变量选择符	Variation Selector
//! FE10	FE1F	竖排形式	Vertical Forms
//! FE20	FE2F	组合用半符号	Combining Half Marks
//! FE30	FE4F	CJK 兼容形式	CJK Compatibility Forms
//! FE50	FE6F	小型变体形式	Small Form Variants
//! FE70	FEFF	阿拉伯表达形式B	Arabic Presentation Form-B
//! FF00	FFEF	半型及全型形式	Halfwidth and Fullwidth Form
//! FFF0	FFFF	特殊	Specials


use std::cmp::Ordering;

const TYPE_NAME: [&str; 145] = ["C1 Control and Latin 1 Supplement", "Latin Extended-A", "Latin Extended-B", "IPA Extensions", "Spacing Modifiers", "Combining Diacritics Marks", "Greek and Coptic", "Cyrillic", "Cyrillic Supplement", "Armenian", "Hebrew", "Arabic", "Syriac", "Arabic Supplement", "Thaana", "N'Ko", "Avestan and Pahlavi", "Mandaic", "Samaritan", "Devanagari", "Bengali", "Gurmukhi", "Gujarati", "Oriya", "Tamil", "Telugu", "Kannada", "Malayalam", "Sinhala", "Thai", "Lao", "Tibetan", "Myanmar", "Georgian", "Hangul Jamo", "Ethiopic", "Ethiopic Supplement", "Cherokee", "Unified Canadian Aboriginal Syllabics", "Ogham", "Runic", "Tagalog", "Hanunoo", "Buhid", "Tagbanwa", "Khmer", "Mongolian", "Cham", "Limbu", "Tai Le", "New Tai Lue", "Kmer Symbols", "Buginese", "Batak", "Lanna", "Balinese", "Sundanese", "Pahawh Hmong", "Lepcha", "Ol Chiki", "Meithei/Manipuri", "Phonetic Extensions", "Phonetic Extensions Supplement", "Combining Diacritics Marks Supplement", "Latin Extended Additional", "Greek Extended", "General Punctuation", "Superscripts and Subscripts", "Currency Symbols", "Combining Diacritics Marks for Symbols", "Letterlike Symbols", "Number Form", "Arrows", "Mathematical Operator", "Miscellaneous Technical", "Control Pictures", "Optical Character Recognition", "Enclosed Alphanumerics", "Box Drawing", "Block Element", "Geometric Shapes", "Miscellaneous Symbols", "Dingbats", "Miscellaneous Mathematical Symbols-A", "Supplemental Arrows-A", "Braille Patterns", "Supplemental Arrows-B", "Miscellaneous Mathematical Symbols-B", "Supplemental Mathematical Operator", "Miscellaneous Symbols and Arrows", "Glagolitic", "Latin Extended-C", "Coptic", "Georgian Supplement", "Tifinagh", "Ethiopic Extended", "Supplemental Punctuation", "CJK Radicals Supplement", "Kangxi Radicals", "Ideographic Description Characters", "CJK Symbols and Punctuation", "Hiragana", "Katakana", "Bopomofo", "Hangul Compatibility Jamo", "Kanbun", "Bopomofo Extended", "CJK Strokes", "Katakana Phonetic Extensions", "Enclosed CJK Letters and Months", "CJK Compatibility", "CJK Unified Ideographs Extension A", "Yijing Hexagrams Symbols", "CJK Unified Ideographs", "Yi Syllables", "Yi Radicals", "Vai", "Unified Canadian Aboriginal Syllabics Supplement", "Modifier Tone Letters", "Latin Extended-D", "Syloti Nagri", "Phags-pa", "Saurashtra", "Javanese", "Chakma", "Varang Kshiti", "Sorang Sompeng", "Newari", "Vit Thai", "Kayah Li", "Hangul Syllables", "High-half zone of UTF-16", "Low-half zone of UTF-16", "Private Use Zone", "CJK Compatibility Ideographs", "Alphabetic Presentation Form", "Arabic Presentation Form-A", "Variation Selector", "Vertical Forms", "Combining Half Marks", "CJK Compatibility Forms", "Small Form Variants", "Arabic Presentation Form-B", "Halfwidth and Fullwidth Form", "Specials"];
const TYPE_RANGE: [(usize, usize);145] = [(0x0080, 0x00FF), (0x0100, 0x017F),(0x0180, 0x024F),(0x0250, 0x02AF),(0x02B0, 0x02FF),(0x0300, 0x036F),(0x0370, 0x03FF),(0x0400, 0x04FF),(0x0500, 0x052F),(0x0530, 0x058F),(0x0590, 0x05FF),(0x0600, 0x06FF),(0x0700, 0x074F),(0x0750, 0x077F),(0x0780, 0x07BF),(0x07C0, 0x07FF),(0x0800, 0x085F),(0x0860, 0x087F),(0x0880, 0x08AF),(0x0900, 0x097F),(0x0980, 0x09FF),(0x0A00, 0x0A7F),(0x0A80, 0x0AFF),(0x0B00, 0x0B7F),(0x0B80, 0x0BFF),(0x0C00, 0x0C7F),(0x0C80, 0x0CFF),(0x0D00, 0x0D7F),(0x0D80, 0x0DFF),(0x0E00, 0x0E7F),(0x0E80, 0x0EFF),(0x0F00, 0x0FFF),(0x1000, 0x109F),(0x10A0, 0x10FF),(0x1100, 0x11FF),(0x1200, 0x137F),(0x1380, 0x139F),(0x13A0, 0x13FF),(0x1400, 0x167F),(0x1680, 0x169F),(0x16A0, 0x16FF),(0x1700, 0x171F),(0x1720, 0x173F),(0x1740, 0x175F),(0x1760, 0x177F),(0x1780, 0x17FF),(0x1800, 0x18AF),(0x18B0, 0x18FF),(0x1900, 0x194F),(0x1950, 0x197F),(0x1980, 0x19DF),(0x19E0, 0x19FF),(0x1A00, 0x1A1F),(0x1A20, 0x1A5F),(0x1A80, 0x1AEF),(0x1B00, 0x1B7F),(0x1B80, 0x1BB0),(0x1BC0, 0x1BFF),(0x1C00, 0x1C4F),(0x1C50, 0x1C7F),(0x1C80, 0x1CDF),(0x1D00, 0x1D7F),(0x1D80, 0x1DBF),(0x1DC0, 0x1DFF),(0x1E00, 0x1EFF),(0x1F00, 0x1FFF),(0x2000, 0x206F),(0x2070, 0x209F),(0x20A0, 0x20CF),(0x20D0, 0x20FF),(0x2100, 0x214F),(0x2150, 0x218F),(0x2190, 0x21FF),(0x2200, 0x22FF),(0x2300, 0x23FF),(0x2400, 0x243F),(0x2440, 0x245F),(0x2460, 0x24FF),(0x2500, 0x257F),(0x2580, 0x259F),(0x25A0, 0x25FF),(0x2600, 0x26FF),(0x2700, 0x27BF),(0x27C0, 0x27EF),(0x27F0, 0x27FF),(0x2800, 0x28FF),(0x2900, 0x297F),(0x2980, 0x29FF),(0x2A00, 0x2AFF),(0x2B00, 0x2BFF),(0x2C00, 0x2C5F),(0x2C60, 0x2C7F),(0x2C80, 0x2CFF),(0x2D00, 0x2D2F),(0x2D30, 0x2D7F),(0x2D80, 0x2DDF),(0x2E00, 0x2E7F),(0x2E80, 0x2EFF),(0x2F00, 0x2FDF),(0x2FF0, 0x2FFF),(0x3000, 0x303F),(0x3040, 0x309F),(0x30A0, 0x30FF),(0x3100, 0x312F),(0x3130, 0x318F),(0x3190, 0x319F),(0x31A0, 0x31BF),(0x31C0, 0x31EF),(0x31F0, 0x31FF),(0x3200, 0x32FF),(0x3300, 0x33FF),(0x3400, 0x4DBF),(0x4DC0, 0x4DFF),(0x4E00, 0x9FBF),(0xA000, 0xA48F),(0xA490, 0xA4CF),(0xA500, 0xA61F),(0xA660, 0xA6FF),(0xA700, 0xA71F),(0xA720, 0xA7FF),(0xA800, 0xA82F),(0xA840, 0xA87F),(0xA880, 0xA8DF),(0xA900, 0xA97F),(0xA980, 0xA9DF),(0xAA00, 0xAA3F),(0xAA40, 0xAA6F),(0xAA80, 0xAADF),(0xAB00, 0xAB5F),(0xAB80, 0xABA0),(0xAC00, 0xD7AF),(0xD800, 0xDBFF),(0xDC00, 0xDFFF),(0xE000, 0xF8FF),(0xF900, 0xFAFF),(0xFB00, 0xFB4F),(0xFB50, 0xFDFF),(0xFE00, 0xFE0F),(0xFE10, 0xFE1F),(0xFE20, 0xFE2F),(0xFE30, 0xFE4F),(0xFE50, 0xFE6F),(0xFE70, 0xFEFF),(0xFF00, 0xFFEF),(0xFFF0, 0xFFFF)];

// 字母字符的范围
const ALPHABETIC_RANGE: [(usize, usize);2] = [(0x0370, 0x07BF), (0x0980, 0x1FFF)];

// 单字字符的范围， 中文（包括日文韩文同用）的范围
const CASED_RANGE: [(usize, usize);5] = [(0x2000, 0x206F), (0x3000, 0x303F), (0x31C0, 0x31EF), (0x3200, 0x9FA5), (0xFF00, 0xFFEF)];


/// 获得字符所在的区间段ID（范围为在1~146）, 返回0为没有找到ID
pub fn get_type_id(c: char) -> usize {
    let c = c as usize;
    if c < 128 {
        return 1;
    }
    if c > 0xffff {
        return 0;
    }
    match TYPE_RANGE.binary_search_by(|&(start, end)| {
        if c < start {
            Ordering::Less
        }else if c > end {
            Ordering::Greater
        }else{
            Ordering::Equal
        }
    }) {
        Ok(i) => i+2,
        _ => 0
    }
}
/// 获得字符所在的区间段ID的名称
pub fn get_type_name(c: char) -> &'static str{
    type_name(get_type_id(c))
}
/// 获得区间段ID的名称
pub fn type_name(id: usize) -> &'static str{
    if id > 1 {
        TYPE_NAME[id - 2]
    }else if id > 0 {
        "ASCII"
    }else{
        ""
    }
}
/// 根据字符所在的语言，判断字符是否为单字字符。 单字字符的范围就是中文（包括日文韩文同用）
pub fn is_cased(c: char) -> bool{
    let c = c as usize;
    match CASED_RANGE.binary_search_by(|&(start, end)| {
        if c < start {
            Ordering::Less
        }else if c > end {
            Ordering::Greater
        }else{
            Ordering::Equal
        }
    }) {
        Ok(_) => true,
        _ => false
    }
}
/// 根据字符所在的语言，判断字符是否为字母字符。 从希腊文到马尔代夫语，孟加拉语到希腊文扩充
pub fn is_alphabetic(c: char) -> bool{
    let c = c as usize;
    if c < 0x0250 {
        return true;
    }
    match ALPHABETIC_RANGE.binary_search_by(|&(start, end)| {
        if c < start {
            Ordering::Less
        }else if c > end {
            Ordering::Greater
        }else{
            Ordering::Equal
        }
    }) {
        Ok(_) => true,
        _ => false
    }

}

/// 定义字符对应代码点的语言区段
pub trait Codepoint where Self: core::marker::Sized {
    /// 获得字符所在的区间段ID（范围为在1~146）, 返回0为没有找到ID
    fn get_type_id(self) -> usize;
    /// 获得字符所在的区间段ID的名称
    fn get_type_name(self) -> &'static str;
    /// 根据字符所在的语言，判断字符是否为单字字符。 单字字符的范围就是中文（包括日文韩文同用）
    fn is_cased(self) -> bool;
    /// 根据字符所在的语言，判断字符是否为字母字符。 范围就是从希腊文到马尔代夫语，孟加拉语到希腊文扩充
    fn is_alpha(self) -> bool;
}
impl Codepoint for char {
    fn get_type_id(self) -> usize{
        get_type_id(self)
    }
    fn get_type_name(self) -> &'static str {
        get_type_name(self)
    }
    fn is_cased(self) -> bool {
        is_cased(self)
    }
    fn is_alpha(self) -> bool {
        is_alphabetic(self)
    }
}

#[test]
fn test_ucd() {
    let c = 'a';
    let c1 = '我';
    let c2 = '장';
    let c3 = 'ρ';
    let c4 = 'A';
    let c5 = 'た';
    
    
    println!("xxxxxxxxxxx:{}", c.is_cased()); 
    println!("xxxxxxxxxxx:{}", c1.is_cased());
    println!("xxxxxxxxxxx:{}", c2.is_cased()); 
    println!("xxxxxxxxxxx:{}", c3.is_cased());
    println!("xxxxxxxxxxx:{}", c4.is_cased());
    println!("xxxxxxxxxxx:{}", c5.is_cased());

    let s = "Löwe 老虎 Léopard";
    assert!(s.is_char_boundary(0));
    // start of `老`
    assert!(s.is_char_boundary(6));
    assert!(s.is_char_boundary(s.len()));

    // second byte of `ö`
    assert!(!s.is_char_boundary(2));

    // third byte of `老`
    assert!(!s.is_char_boundary(8));
    
}

