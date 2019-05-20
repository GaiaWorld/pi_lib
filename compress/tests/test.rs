extern crate compress;

use compress::{CompressLevel, compress, uncompress};

#[test]
fn test_lz4() {
    let string = String::from("asdfasdfpoiq;'wej(*(^$l;kKJ个）（&（）KLJ：LJLK：JLK：J：）（*）（*&（*&……&%……*UJK《JJL：HKLJHLKJHKJHKLHL：L：KHKJLGHYU……*&（%&……￥R%$#%$@#$%EDGFVNMLI_)(*%ERDHJGH0907886rtfhh)(&&$%$GFJHHJLJIOP(*jg%&$oujhlkjhnmgjhgljy98^&%^##$@$9878756543jkhmnbkmjou(*&(%^%$dfdhgjnlku^^%$$#%$egfcvmjhnl:kjo(&(&^%^%erfdgbh<jhkhiu^*(&*%&^%$^%ergfghghjlnbcvvxdasaew#$#%^*()_)(ytghjkl<mn%^%%#$%erdcffv:+{?}*&^%$#@!wsdefgw@@#$%^&JK;IO[IOU9078965(*&^%#$%$TGHJGFDFDGJHKIUTyghjkhty&ytkjljhgfghjhgfcvbnmrt(*&#*^$#^&*(*&^%$%&*(*&^%&^%$yhgffvbnmikyr$##%^&*(*&阿斯利康大家法律萨芬基本原理；声嘶力竭j8aslkjdfqpkmvpo09模压暗室逢灯阿斯顿发生地方东奔西走；辊；；基金会利用好吗，民");
    let buffer = string.as_bytes();
    let mut vec = Vec::with_capacity(0);
    vec.reserve(800);
    vec.resize(800, 0);
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!buffer len: {}, vec len: {}, vec: {:?}", buffer.len(), vec.len(), vec);
    assert!(compress(buffer, &mut vec, CompressLevel::High).is_ok());
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!vec len: {}, vec: {:?}", vec.len(), vec);

    let mut vec_ = Vec::with_capacity(0);
    vec_.reserve(800);
    vec_.resize(800, 0);
    assert!(uncompress(&vec[..], &mut vec_).is_ok());
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!vec_ len: {}, vec_: {:?}", vec_.len(), vec_);
    assert!(String::from_utf8(vec_).ok().unwrap() == string);
}