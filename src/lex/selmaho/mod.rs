mod classify;
mod display;

/// The classification of a word.
///
/// Most of the variants conform to the strict meaning of selmaho, which is the grammatical type of a cmavo.
/// A few others represent other word types in Lojban: `Cmevla`, `Gismu`, `Fuhivla`, and `Lujvo`.
/// Finally, there are some "technical" selmaho: `AnyText`, `UnknownCmavo`, and `ZoiDelimiter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Selmaho {
	// regular cmavo
	A,
	Bai,
	Bahe,
	Be,
	Bei,
	Beho,
	Bihe,
	Bihi,
	Bo,
	Boi,
	Bu,
	By,
	Cai,
	Caha,
	Cei,
	Cehe,
	Co,
	Coi,
	Cu,
	Cuhe,
	Daho,
	Doi,
	Dohu,
	Fa,
	Faha,
	Faho,
	Fehe,
	Fehu,
	Fiho,
	Foi,
	Fuha,
	Fuhe,
	Fuho,
	Ga,
	Gaho,
	Gehu,
	Gi,
	Giha,
	Goi,
	Goha,
	Guha,
	I,
	Ja,
	Jai,
	Joi,
	Johi,
	Ke,
	Kei,
	Kehe,
	Ki,
	Koha,
	Ku,
	Kuhe,
	Kuho,
	La,
	Lau,
	Lahe,
	Le,
	Lehu,
	Li,
	Lihu,
	Loho,
	Lohu,
	Lu,
	Luhu,
	Mai,
	Maho,
	Me,
	Mehu,
	Moi,
	Mohe,
	Mohi,
	Na,
	Nai,
	Nahe,
	Nahu,
	Nihe,
	Niho,
	Noi,
	Nu,
	Nuha,
	Nuhi,
	Nuhu,
	Pa,
	Pehe,
	Peho,
	Pu,
	Raho,
	Roi,
	Sa,
	Se,
	Sei,
	Sehu,
	Si,
	Soi,
	Su,
	Tahe,
	Tei,
	Tehu,
	To,
	Toi,
	Tuhe,
	Ui,
	Va,
	Vau,
	Vei,
	Veha,
	Veho,
	Viha,
	Vuho,
	Vuhu,
	Xi,
	Y,
	Zaho,
	Zei,
	Zeha,
	Zi,
	Zihe,
	Zo,
	Zoi,
	Zohu,

	// experimental selmaho (selmaho that contain only experimental cmavo)
	Bahei,
	Beihe,
	Boihoi,
	Boihohu,
	Cahei,
	Ceheihoi,
	Cohai,
	Cohehohe,
	Cohuho, // treated like CO
	Cuhau,
	Dauho, // treated like UI
	Dehai,
	Dehau, // treated like UI
	Dohoi,
	Fauha,  // treated like UI
	Fahoho, // treated like FAhO
	Fihoi,
	Foihe,
	Gahuhau,
	Gehuhi,
	Gihei,
	Gihoi,
	Gohoi,
	Ihau,
	Jaiha,
	Jaihi,
	Jauhu,
	Jahoi,
	Jihoi,
	Joihi,
	Johe,
	Juhau,
	Juhei,
	Juhuhi,
	Kauhai,
	Kauhu,
	Keihau,
	Keihi,
	Kehai,
	Kehei,
	Keheiha,
	Kehehau,
	Kehehu,
	Kehuhi,
	Kuhau,
	Kuhei,
	Kuhoihu,
	Kyhoi,
	Lehai,
	Lihau,
	Lihei,
	Lohai,
	Lohoi,
	Luhei,
	Mauhau,
	Mauhe,
	Mauho,
	Meihe,
	Meiho,
	Mehoi,
	Muhoi,
	Muhohu,
	Neihai,
	Noiha,
	Noihau,
	Noihahu,
	Noihi,
	Nohoi,
	Rauho,
	Rehaihe,
	Sauhu,
	Sahau,
	Sahoi,
	Seihau,
	Seiho,
	Sehehi,
	Sehoihoi,
	Sihihei,
	Sihihoi,
	Sihoi,
	Soihi,
	Sohehai,
	Sohoi,
	Taihu,
	Tauho,
	Tahoi,
	Tahuhi,
	Teihu,
	Tehoihoi,
	Toihe,
	Toiho,
	Vauhehoi,
	Vauhohoi,
	Vuhoi,
	Vyhy,
	Xauhe,
	Xauheho,
	Xauho,
	Xauhoi,
	Xauhoho,
	Xahoihahoiha, // treated like an error
	Xehau,
	Xoi,
	Xoha,
	Xohehohe,
	Xohi,
	Xuhau,
	Yhi,
	Zaihai,
	Zaihu,
	Zauhehu,
	Zeihei,
	// in jbovlaste as ZEI'OI (yes, that's a lowercase apostrophe in a selmaho)
	Zeihoi,
	Ziheha,
	Zihehau,
	Zihoi,
	Zoihai,
	Zoihohe,
	Zohau,
	Zohehu,
	Zohiho,
	Zohoi,

	// non-cmavo selmaho

	// the first three are are brivla and as such are treated the same by parsers, but the distinction may be helpful for other users of the lexer
	Gismu,
	Fuhivla,
	Lujvo,
	Cmevla,

	// technical selmaho
	/// For words that have cmavo form but aren't recognized as a specific selmaho
	UnknownCmavo,
	/// Whenever the text is not recognized. May occur in valid Lojban, between, for example, the delimiters of `zoi` constructs, but not necessarily.
	AnyText,
	/// Only emitted by the lexer so not technically a selmaho
	ZoiDelimiter,
}

impl Selmaho {
	/// If the selmaho itself is experimental, meaning that all the cmavo in it are experimental.
	/// False for all non-cmavo and technical cmavo, except `UnknownCmavo`.
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::lex::selmaho::Selmaho;
	/// assert!(!Selmaho::A.is_fundamentally_experimental());
	/// assert!(Selmaho::Xahoihahoiha.is_fundamentally_experimental());
	/// assert!(!Selmaho::Gismu.is_fundamentally_experimental());
	/// assert!(!Selmaho::ZoiDelimiter.is_fundamentally_experimental());
	/// ```
	#[must_use]
	#[allow(clippy::too_many_lines)]
	pub fn is_fundamentally_experimental(self) -> bool {
		// brevity is sacrificed on the altar of exhaustiveness
		match self {
			Self::Bahei
			| Self::Beihe
			| Self::Boihoi
			| Self::Boihohu
			| Self::Cahei
			| Self::Ceheihoi
			| Self::Cohai
			| Self::Cohehohe
			| Self::Cohuho
			| Self::Cuhau
			| Self::Dauho
			| Self::Dehai
			| Self::Dehau
			| Self::Dohoi
			| Self::Fauha
			| Self::Fahoho
			| Self::Fihoi
			| Self::Foihe
			| Self::Gahuhau
			| Self::Gehuhi
			| Self::Gihei
			| Self::Gihoi
			| Self::Gohoi
			| Self::Ihau
			| Self::Jaiha
			| Self::Jaihi
			| Self::Jauhu
			| Self::Jahoi
			| Self::Jihoi
			| Self::Joihi
			| Self::Johe
			| Self::Juhau
			| Self::Juhei
			| Self::Juhuhi
			| Self::Kauhai
			| Self::Kauhu
			| Self::Keihau
			| Self::Keihi
			| Self::Kehai
			| Self::Kehei
			| Self::Keheiha
			| Self::Kehehau
			| Self::Kehehu
			| Self::Kehuhi
			| Self::Kuhau
			| Self::Kuhei
			| Self::Kuhoihu
			| Self::Kyhoi
			| Self::Lehai
			| Self::Lihau
			| Self::Lihei
			| Self::Lohai
			| Self::Lohoi
			| Self::Luhei
			| Self::Mauhau
			| Self::Mauhe
			| Self::Mauho
			| Self::Meihe
			| Self::Meiho
			| Self::Mehoi
			| Self::Muhoi
			| Self::Muhohu
			| Self::Neihai
			| Self::Noiha
			| Self::Noihau
			| Self::Noihahu
			| Self::Noihi
			| Self::Nohoi
			| Self::Rauho
			| Self::Rehaihe
			| Self::Sauhu
			| Self::Sahau
			| Self::Sahoi
			| Self::Seihau
			| Self::Seiho
			| Self::Sehehi
			| Self::Sehoihoi
			| Self::Sihihei
			| Self::Sihihoi
			| Self::Sihoi
			| Self::Soihi
			| Self::Sohehai
			| Self::Sohoi
			| Self::Taihu
			| Self::Tauho
			| Self::Tahoi
			| Self::Tahuhi
			| Self::Teihu
			| Self::Tehoihoi
			| Self::Toihe
			| Self::Toiho
			| Self::Vauhehoi
			| Self::Vauhohoi
			| Self::Vuhoi
			| Self::Vyhy
			| Self::Xauhe
			| Self::Xauheho
			| Self::Xauho
			| Self::Xauhoi
			| Self::Xauhoho
			| Self::Xahoihahoiha
			| Self::Xehau
			| Self::Xoi
			| Self::Xoha
			| Self::Xohehohe
			| Self::Xohi
			| Self::Xuhau
			| Self::Yhi
			| Self::Zaihai
			| Self::Zaihu
			| Self::Zauhehu
			| Self::Zeihei
			| Self::Zeihoi
			| Self::Ziheha
			| Self::Zihehau
			| Self::Zihoi
			| Self::Zoihai
			| Self::Zoihohe
			| Self::Zohau
			| Self::Zohehu
			| Self::Zohiho
			| Self::Zohoi
			| Self::UnknownCmavo => true,
			Self::A
			| Self::Bai
			| Self::Bahe
			| Self::Be
			| Self::Bei
			| Self::Beho
			| Self::Bihe
			| Self::Bihi
			| Self::Bo
			| Self::Boi
			| Self::Bu
			| Self::By
			| Self::Cai
			| Self::Caha
			| Self::Cei
			| Self::Cehe
			| Self::Co
			| Self::Coi
			| Self::Cu
			| Self::Cuhe
			| Self::Daho
			| Self::Doi
			| Self::Dohu
			| Self::Fa
			| Self::Faha
			| Self::Faho
			| Self::Fehe
			| Self::Fehu
			| Self::Fiho
			| Self::Foi
			| Self::Fuha
			| Self::Fuhe
			| Self::Fuho
			| Self::Ga
			| Self::Gaho
			| Self::Gehu
			| Self::Gi
			| Self::Giha
			| Self::Goi
			| Self::Goha
			| Self::Guha
			| Self::I
			| Self::Ja
			| Self::Jai
			| Self::Joi
			| Self::Johi
			| Self::Ke
			| Self::Kei
			| Self::Kehe
			| Self::Ki
			| Self::Koha
			| Self::Ku
			| Self::Kuhe
			| Self::Kuho
			| Self::La
			| Self::Lau
			| Self::Lahe
			| Self::Le
			| Self::Lehu
			| Self::Li
			| Self::Lihu
			| Self::Loho
			| Self::Lohu
			| Self::Lu
			| Self::Luhu
			| Self::Mai
			| Self::Maho
			| Self::Me
			| Self::Mehu
			| Self::Moi
			| Self::Mohe
			| Self::Mohi
			| Self::Na
			| Self::Nai
			| Self::Nahe
			| Self::Nahu
			| Self::Nihe
			| Self::Niho
			| Self::Noi
			| Self::Nu
			| Self::Nuha
			| Self::Nuhi
			| Self::Nuhu
			| Self::Pa
			| Self::Pehe
			| Self::Peho
			| Self::Pu
			| Self::Raho
			| Self::Roi
			| Self::Sa
			| Self::Se
			| Self::Sei
			| Self::Sehu
			| Self::Si
			| Self::Soi
			| Self::Su
			| Self::Tahe
			| Self::Tei
			| Self::Tehu
			| Self::To
			| Self::Toi
			| Self::Tuhe
			| Self::Ui
			| Self::Va
			| Self::Vau
			| Self::Vei
			| Self::Veha
			| Self::Veho
			| Self::Viha
			| Self::Vuho
			| Self::Vuhu
			| Self::Xi
			| Self::Y
			| Self::Zaho
			| Self::Zei
			| Self::Zeha
			| Self::Zi
			| Self::Zihe
			| Self::Zo
			| Self::Zoi
			| Self::Zohu
			| Self::Gismu
			| Self::Fuhivla
			| Self::Lujvo
			| Self::Cmevla
			| Self::AnyText
			| Self::ZoiDelimiter => false,
		}
	}
}
