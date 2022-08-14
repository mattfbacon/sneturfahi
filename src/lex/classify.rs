use super::token::Selmaho;
use crate::rules::ParseResultExt;

impl Selmaho {
	/// Also returns if the word is experimental (only for cmavo)
	pub fn classify(word: &str) -> (Self, bool) {
		// XXX: evaluate performance of alternative options. maybe a stack-allocated `str` with the size of the longest specifically recognized cmavo?
		let word: String = word
			.chars()
			.filter(|&ch| ch != ',')
			.map(|ch| match ch {
				'h' => '\'',
				other => other,
			})
			.collect();

		let mut is_experimental = false;

		macro_rules! experimental {
			($val:expr) => {{
				is_experimental = true;
				$val
			}};
		}

		let selmaho = match word.as_str() {
			"a" | "e" | "ji" | "o" | "u" => Self::A,
			"e'u'a" | "i'a'a" => experimental!(Self::A),
			"ba'i" | "bai" | "bau" | "be'i" | "ca'i" | "cau" | "ci'e" | "ci'o" | "ci'u" | "cu'u"
			| "de'i" | "di'o" | "du'i" | "du'o" | "fa'e" | "fi'e" | "ga'a" | "gau" | "ja'e" | "ji'e"
			| "ji'o" | "ji'u" | "ka'a" | "ka'i" | "kai" | "ki'i" | "ki'u" | "koi" | "ku'u" | "la'u"
			| "le'a" | "li'e" | "ma'e" | "ma'i" | "mau" | "me'a" | "me'e" | "mu'i" | "mu'u" | "ni'i"
			| "pa'a" | "pa'u" | "pi'o" | "pu'a" | "pu'e" | "ra'a" | "ra'i" | "rai" | "ri'a" | "ri'i"
			| "sau" | "si'u" | "ta'i" | "tai" | "ti'i" | "ti'u" | "tu'i" | "va'o" | "va'u" | "zau"
			| "zu'e" => Self::Bai,
			"be'ei" | "da'ai'a" | "de'i'a" | "de'i'e" | "de'i'i" | "de'i'o" | "de'i'u" | "fi'ei"
			| "gai'i" | "ja'u" | "ka'ai" | "kai'ai" | "ki'oi" | "ko'au" | "ku'ai" | "li'i'e"
			| "mu'ai" | "mu'e'ei" | "nai'i" | "nu'ai" | "pau'u" | "po'a" | "pu'ai" | "te'a'a"
			| "te'i" | "ti'u'a" | "ti'u'e" | "ti'u'i" | "xau" => experimental!(Self::Bai),
			"ba'e" | "za'e" | "ci'a" => Self::Bahe,
			"pe'ei" | "zai'e" | "zei'e" => experimental!(Self::Bahe),
			"be" => Self::Be,
			"bei" => Self::Bei,
			"be'o" => Self::Beho,
			"bi'e" => Self::Bihe,
			"bi'i" | "bi'o" | "mi'i" => Self::Bihi,
			"bo" => Self::Bo,
			"boi" => Self::Boi,
			"boi'e'u" => experimental!(Self::Boi),
			"bu" => Self::Bu,
			"by" | "cy" | "dy" | "fy" | "ga'e" | "ge'o" | "gy" | "je'o" | "jo'o" | "jy" | "ky"
			| "lo'a" | "ly" | "my" | "na'a" | "ny" | "py" | "ru'o" | "ry" | "se'e" | "sy" | "to'a"
			| "ty" | "vy" | "xy" | "y'y" | "zy" => Self::By,
			"bu'o'e" | "e'y" | "i'y" | "iy" | "iy'y" | "o'y" | "u'y" | "uy" | "jo'au'o" | "ro'au'o" => {
				experimental!(Self::By)
			}
			"cai" | "cu'i" | "pei" | "ru'e" | "sai" => Self::Cai,
			"cau'i" | "dai'i" | "na'oi" | "ni'au" | "pei'a" | "dau'i" | "mau'i" | "me'ai" => {
				experimental!(Self::Cai)
			}
			"ca'a" | "ka'e" | "nu'o" | "pu'i" => Self::Caha,
			"bi'ai" => experimental!(Self::Caha),
			"cei" => Self::Cei,
			"ce'e" => Self::Cehe,
			"co" => Self::Co,
			"co'ai'e" | "co'au'e" | "co'o'e" => experimental!(Self::Co),
			"be'e" | "coi" | "co'o" | "fe'o" | "fi'i" | "je'e" | "ju'i" | "ke'o" | "ki'e" | "mi'e"
			| "mu'o" | "nu'e" | "pe'u" | "re'i" | " ta'a" | "vi'o" => Self::Coi,
			"a'oi" | "bu'oi" | "co'oi" | "de'a'ai" | "di'a'ai" | "di'ai" | "doi'oi" | "fau'u"
			| "fe'oi" | "fi'i'e" | "ke'o'a" | "ke'o'o" | "ke'o'u" | "ku'au'i" | "ku'o'e'a" | "o'ai"
			| "sau'ei" | "sau'e'u" | "sei'ai" | "sei'u" | "te'ei" | "ve'ei" | "xu'e" | "zau'e"
			| "ci'oi" | "go'au" | "goi'e" | "jo'au" | "ki'ai" | "sa'ei" | "tai'i" => {
				experimental!(Self::Coi)
			}
			"cu" => Self::Cu,
			"cu'e" | "nau" => Self::Cuhe,
			"ba'au" | "pu'au" => experimental!(Self::Cuhe),
			"da'o" => Self::Daho,
			"dai'o" | "do'ai" | "xei'a" | "xei'i" | "xei'u" => experimental!(Self::Daho),
			"doi" => Self::Doi,
			"ve'ai" | "da'ei" | "da'oi" => experimental!(Self::Doi),
			"do'u" => Self::Dohu,
			"fa" | "fai" | "fe" | " fi" | "fi'a" | "fo" | "fu" => Self::Fa,
			"fai'i" | "fa'au'u" | "zoi'u" => experimental!(Self::Fa),
			"be'a" | "vu'a" | "du'a" | "ne'u" | "ru'u" | "ri'u" | "ti'a" | "ga'u" | "ca'u" | "ni'a"
			| "zu'a" | "re'o" | "te'e" | "bu'u" | "ne'a" | "ne'i" | "pa'o" | "to'o" | "fa'a" | "ze'o"
			| "zo'a" | "zo'i" => Self::Faha,
			"bau'u" | "gau'o" | "du'oi" | "zu'au" => experimental!(Self::Faha),
			"fa'o" => Self::Faho,
			"fa'u'u'u'u'u'u'u'u" | "to'au" => experimental!(Self::Faho),
			"fe'e" => Self::Fehe,
			"fi'o" => Self::Fiho,
			"foi" => Self::Foi,
			"fu'a" => Self::Fuha,
			"fu'e" => Self::Fuhe,
			"fu'ei" => experimental!(Self::Fuhe),
			"fu'o" => Self::Fuho,
			"ga" | "ge" | "ge'i" | "go" | "gu" => Self::Ga,
			"ge'u'a" | "gi'a'a" => experimental!(Self::Ga),
			"ga'o" | "ke'i" => Self::Gaho,
			"xai'u'oi" | "xoi'u'oi" | "ma'a'u'oi" | "xau'u'oi" | "xei'u'oi" => {
				experimental!(Self::Gaho)
			}
			"ge'u" => Self::Gehu,
			"gi" => Self::Gi,
			"gi'a" | "gi'e" | "gi'i" | "gi'o" | "gi'u" => Self::Giha,
			"gi'e'u'a" | "gi'i'a'a" => experimental!(Self::Giha),
			"goi" | "po" | "po'e" | "po'u" | "ne" | "no'u" | "pe" => Self::Goi,
			"voi'e" => experimental!(Self::Goi),
			"go'a" | "go'e" | "go'i" | "go'o" | "go'u" | "bu'a" | "bu'e" | "bu'i" | "co'e" | "du"
			| "mo" | "nei" | "no'a" => Self::Goha,
			"gai'o" | "gi'o'i" | "go'ai" | "ku'ai'i" | "cei'i" | "xe'u" => experimental!(Self::Goha),
			"gu'a" | "gu'e" | "gu'i" | "gu'o" | "gu'u" => Self::Guha,
			"gu'e'u'a" | "gu'i'a'a" => experimental!(Self::Guha),
			"i" => Self::I,
			"ja" | "je" | "je'i" | "jo" | "ju" => Self::Ja,
			"je'u'a" | "ji'a'a" => experimental!(Self::Ja),
			"jai" => Self::Jai,
			"ja'ei" | "jai'e" | "jo'ai" => experimental!(Self::Jai),
			"joi" | "fa'u" | "pi'u" | "jo'e" | "jo'u" | "ju'e" | "ku'a" | "ce" | "ce'o" => Self::Joi,
			"fa'u'ai" | "xoi'u" | "jo'ei" | "jo'ei'i" | "joi'au'a" | "jo'oi" | "bo'a'oi" | "ce'au"
			| "ce'oi" => experimental!(Self::Joi),
			"jo'i" => Self::Johi,
			"ke" => Self::Ke,
			"va'au" | "fei'u" | "ke'oi" | "pi'ai" => experimental!(Self::Ke),
			"kei" => Self::Kei,
			"ki" => Self::Ki,
			"da" | "de" | "di" | "da'u" | "de'e" | "dei" | "de'u" | "di'e" | "da'e" | "di'u" | "do'i"
			| "ko" | "ma'a" | "mi" | "mi'a" | "do" | "do'o" | "mi'o" | "vo'a" | "vo'e" | "vo'i"
			| "vo'o" | "vo'u" | "fo'a" | "fo'e" | "fo'i" | "fo'o" | "fo'u" | "ko'a" | "ko'e" | "ko'i"
			| "ko'o" | "ko'u" | "ra" | "ru" | "ri" | "ti" | "tu" | "ta" | "ke'a" | "ma" | "zi'o"
			| "zo'e" | "zu'i" | "ce'u" => Self::Koha,
			"kau'a" | "kau'e" | "kau'i" | "da'au" | "lau'e" | "lau'u" | "dei'ei" | "ra'au" | "da'ai"
			| "do'o'o" | "mi'ai" | "mi'oi" | "vau'a" | "vau'e" | "vau'o" | "vau'u" | "ri'au" | "xai"
			| "zoi'i" | "tu'oi" | "tu'oi'u" | "ca'au" | "di'au" | "di'ei" | "di'oi" | "do'au"
			| "do'ei" | "mai'i" | "nau'o" | "nau'u" | "zai'o" | "sy'y" | "zu'ai" | "bo'i" | "bo'o"
			| "bo'u" | "bo'a" | "bo'e" => experimental!(Self::Koha),
			"ku" => Self::Ku,
			"ku'e" => Self::Kuhe,
			"ku'o" => Self::Kuho,
			"la" | "la'i" | "lai" => Self::La,
			"lai'u" | "la'ei" => experimental!(Self::La),
			"lau" | "tau" | "zai" | "ce'a" => Self::Lau,
			"la'e" | "tu'a" | "vu'i" | "lu'a" | "lu'e" | "lu'i" | "lu'o" => Self::Lahe,
			"tau'e" | "lai'e" | "la'e'au" | "lu'au" | "cei'u" | "du'au" | "moi'a" | "zo'ei" => {
				experimental!(Self::Lahe)
			}
			"le'u" => Self::Lehu,
			"li" | "me'o" => Self::Li,
			"li'ai" | "na'au" => experimental!(Self::Li),
			"li'u" => Self::Lihu,
			"lo'o" => Self::Loho,
			"mai" | "mo'o" => Self::Mai,
			"sai'ei" | "ju'ai" | "lai'a" | "ba'ai" => experimental!(Self::Mai),
			"ma'o" => Self::Maho,
			"me" => Self::Me,
			"du'ai" | "me'au" | "mei'u" => experimental!(Self::Me),
			"me'u" => Self::Mehu,
			"moi" | "si'e" | "va'e" | "mei" | "cu'o" => Self::Moi,
			"jei'o" | "ka'oi" | "lei'o" | "cei'a" | "cu'oi'e" | "coi'e" | "doi'e" | "mei'i" | "moi'e"
			| "moi'o" | "moi'u" | "nei'o" | "soi'e" => experimental!(Self::Moi),
			"mo'e" => Self::Mohe,
			"mo'i" => Self::Mohi,
			"na" | "ja'a" => Self::Na,
			"xa'au" | "xu'o'e" | "cau'a" | "mai'a" | "mai'e" | "na'ai" => experimental!(Self::Na),
			"nai" => Self::Nai,
			"ja'ai" => experimental!(Self::Nai),
			"na'e" | "to'e" | "je'a" | "no'e" => Self::Nahe,
			"rai'a" | "rei'e" | "sai'e" | "gu'y" | "je'ai" | "cai'e" | "cau'e" | "cau'o'e" | "na'ei"
			| "ni'u'u" | "noi'e" | "pai'e" => experimental!(Self::Nahe),
			"na'u" => Self::Nahu,
			"ni'e" => Self::Nihe,
			"ni'o" | "no'i" => Self::Niho,
			"noi" | "poi" | "voi" => Self::Noi,
			"voi'i" => experimental!(Self::Noi),
			"nu" | "si'o" | "jei" | "ka" | "li'i" | "du'u" | "ni" | "su'u" | "pu'u" | "za'i" | "mu'e"
			| "zu'o" => Self::Nu,
			"poi'i" | "te'oi" | "ga'ei" | "ka'ei" | "kai'ei" | "kai'u" | "ka'oi'i" | "bu'ai"
			| "ni'ai" | "xe'ei" => experimental!(Self::Nu),
			"nu'a" => Self::Nuha,
			"nu'i" => Self::Nuhi,
			"nu'u" => Self::Nuhu,
			"pa" | "re" | "ci" | "vo" | "mu" | "xa" | "ze" | "bi" | "so" | "no" | "rei" | "fei"
			| "vai" | "gai" | "jau" | "dau" | "fi'u" | "ra'e" | "za'u" | "ki'o" | "me'i" | "ma'u"
			| "ce'i" | "ni'u" | "pi" | "pi'e" | "rau" | "ro" | "xo" | "ji'i" | "da'a" | "du'e"
			| "mo'a" | "so'a" | "so'e" | "so'i" | "so'o" | "so'u" | "su'o" | "te'o" | "tu'o" | "ka'o"
			| "ci'i" | "no'o" | "pai" => Self::Pa,
			"kei'ei" | "lai'ai" | "xei" | "sei'a" | "si'ei" | "ro'oi" | "xo'e" | "ja'au" | "ku'i'a"
			| "mei'a" | "no'ai" | "no'e'u" | "zau'u" | "ze'au" | "xe'e" | "so'au" | "su'oi" | "su'ai"
			| "su'au" | "fai'e'au" | "fai'u" | "fai'u'a" | "pu'e'u'o" | "se'i'i" | "sei'u'e"
			| "tau'u" | "vai'ei'a" | "vau'au'o" | "vo'ei'a" | "fu'a'ai" | "fu'a'au" | "ga'au"
			| "gau'i'o" | "go'o'i'a" | "ka'ei'a" | "kai'o" | "ka'o'ai" | "ka'o'ei" | "kau'o"
			| "kei'o" | "koi'o" | "mai'e'e" | "mu'i'ai" | "na'a'au" | "ni'e'ei" | "ni'e'oi"
			| "pa'au'o" | "pei'i'a" | "vi'ei'e" | "pi'au" | "xi'i'ei" => experimental!(Self::Pa),
			"pe'e" => Self::Pehe,
			"pe'o" => Self::Peho,
			"kei'ai" => experimental!(Self::Peho),
			"pu" | "ba" | "ca" => Self::Pu,
			"xa'ei" => experimental!(Self::Pu),
			"ra'o" => Self::Raho,
			"roi" | "re'u" => Self::Roi,
			"va'ei" | "ba'oi" | "de'ei" | "mu'ei" => experimental!(Self::Roi),
			"sa" => Self::Sa,
			"se" | "te" | "ve" | "xe" => Self::Se,
			"re'au'e" | "se'ai'e" | "se'au'e" | "se'o'e" | "se'u'o" | "to'ai" | "tu'ei" | "vo'ai"
			| "xo'ai" | "ko'ei" | "lu'oi" | "ze'ai'e" | "ze'au'e" | "so'o'o'oi" | "su'ei" => {
				experimental!(Self::Se)
			}
			"sei" | "ti'o" => Self::Sei,
			"sei'e" | "le'au" | "cei'e" => experimental!(Self::Sei),
			"se'u" => Self::Sehu,
			"si" => Self::Si,
			"si'au'i" | "ze'ei" => experimental!(Self::Si),
			"soi" => Self::Soi,
			"su" => Self::Su,
			"ta'e" | "ru'i" | "di'i" | "na'o" => Self::Tahe,
			"dei'a" | "ze'ai" | "zei'a" => experimental!(Self::Tahe),
			"tei" => Self::Tei,
			"te'u" => Self::Tehu,
			"to" | "to'i" => Self::To,
			"toi" => Self::Toi,
			"tu'e" => Self::Tuhe,
			"a'a" | "a'e" | "a'i" | "a'o" | "a'u" | "au" | "i'a" | "ia" | "i'e" | "ie" | "ua" | "u'e"
			| "u'i" | "ue" | "ui" | "u'o" | "uo" | "u'u" | "uu" | "i'i" | "ii" | "i'o" | "io" | "i'u"
			| "iu" | "e'a" | "e'e" | "e'i" | "ei" | "e'o" | "e'u" | "o'a" | "o'e" | "o'i" | "oi"
			| "o'o" | "o'u" | "ru'a" | "se'o" | "ti'e" | "za'a" | "ju'a" | "ka'u" | "ba'a" | "ca'e"
			| "pe'i" | "su'a" | "ba'u" | "ra'u" | "sa'e" | "sa'u" | "ta'o" | " to'u" | "va'i"
			| "je'u" | "ke'u" | "la'a" | "li'a" | "da'i" | "do'a" | "mu'a" | "pa'e" | "zu'u" | "re'e"
			| "ro'a" | "ro'e" | "ro'i" | "ro'o" | "ro'u" | "ri'e" | "se'a" | "be'u" | "se'i" | "vu'e"
			| "fu'i" | "ga'i" | "ju'o" | "le'o" | "dai" | "zo'o" | "xu" | "ge'e" | "ki'a" | "bu'o"
			| "xu'o" => Self::Ui,
			"u'ai" | "uai" | "uau" | "ue'i" | "ui'i" | "uo'o" | "uu'i" | "ia'u" | "ie'e" | "ie'i"
			| "a'au" | "ai'i" | "au'u" | "ci'au'u'au'i" | "e'ei" | "ei'au" | "ei'e" | "ei'i"
			| "mau'u" | "mi'au" | "ne'au" | "oi'o" | "oi'u" | "ri'ai" | "sei'i" | "si'au" | "jei'u"
			| "ju'oi" | "kai'a" | "kai'e" | "ki'au" | "lai'i" | "moi'i" | "pei'e" | "za'au" | "vo'oi"
			| "vei'i" | "ta'ei" | "te'i'o" | "xa'i" | "xu'a" | "fu'au" | "je'au" | "so'ei" | "sy'a"
			| "xe'o" | "roi'i" | "pei'o" | "ra'i'au" | "xo'o" | "jau'i" | "ji'ei" | "bo'oi" | "dai'a"
			| "noi'u" | "zai'a" | "ki'a'au'u'au'i" | "ko'oi" => experimental!(Self::Ui),
			"va" | "vi" | "vu" => Self::Va,
			"xa'e" => experimental!(Self::Va),
			"vau" => Self::Vau,
			"vei" => Self::Vei,
			"ve'a" | "ve'e" | "ve'i" | "ve'u" => Self::Veha,
			"ve'o" => Self::Veho,
			"ve'oi" => experimental!(Self::Veho),
			"vi'a" | "vi'u" | "vi'e" | "vi'i" => Self::Viha,
			"vu'o" => Self::Vuho,
			"xi" => Self::Xi,
			"fau'e" | "te'ai" | "xi'e" | "xi'i" => experimental!(Self::Xi),
			word if word.len() >= 1 && word.chars().all(|ch| ch == 'y') => Self::Y,
			"ie'o" | "ko'o'o'o'o" => experimental!(Self::Y),
			"za'o" | "ba'o" | "pu'o" | "ca'o" | "co'a" | "co'i" | "co'u" | "de'a" | "di'a" | "mo'u" => {
				Self::Zaho
			}
			"sau'a" | "xo'u" | "ca'o'a" | "co'a'a" | "co'au'a" | "co'u'a" | "xa'o" => {
				experimental!(Self::Zaho)
			}
			"zei" => Self::Zei,
			"ze'a" | "ze'e" | "ze'i" | "ze'u" => Self::Zeha,
			"zei'au" => experimental!(Self::Zeha),
			"zi" | "za" | "zu" => Self::Zi,
			"za'ai" => experimental!(Self::Zi),
			"zi'e" => Self::Zihe,
			"zo" => Self::Zo,
			"ra'ai" | "doi'u" | "ma'oi" | "ma'oi'e" => experimental!(Self::Zo),
			"zoi" | "la'o" => Self::Zoi,
			"zo'u" => Self::Zohu,
			"fi'ai" | "ge'ai" | "ke'au" | "ce'ai" => experimental!(Self::Zohu),

			// experimental selmaho
			"ba'ei" => Self::Bahei,
			"bei'e" => Self::Beihe,
			"boi'oi" => Self::Boihoi,
			"ca'ei" | "pu'ei" => Self::Cahei,
			"ce'ei'oi" => Self::Ceheihoi,
			"co'ai" => Self::Cohai,
			"co'e'o'e" => Self::Cohehohe,
			"co'u'o" => Self::Cohuho,
			"cu'au" => Self::Cuhau,
			"dau'o" => Self::Dauho,
			"de'ai" => Self::Dehai,
			"dau'a" | "de'au" | "de'oi" | "doi'a" => Self::Dehau,
			"do'oi" => Self::Dohoi,
			"fau'a" | "fau'ai" => Self::Fauha,
			"fa'o'o" => Self::Fahoho,
			"fi'oi" => Self::Fihoi,
			"foi'e" => Self::Foihe,
			"ga'u'au" | "ni'a'a" => Self::Gahuhau,
			"ge'u'i" => Self::Gehuhi,
			"gi'ei" => Self::Gihei,
			"gi'oi" => Self::Gihoi,
			"bo'ei" | "go'oi" | "sau'e" | "ta'ai" | "ze'oi" => Self::Gohoi,
			"iau" | "i'au" => Self::Ihau,
			"jai'a" => Self::Jaiha,
			"jai'i" => Self::Jaihi,
			"jau'u" => Self::Jauhu,
			"fau'au" | "ja'oi" => Self::Jahoi,
			"ji'oi" | "ni'oi" => Self::Jihoi,
			"boi'au" | "fa'ei" | "gei'i'e" => Self::Joihi,
			"xa'ei'o" | "xa'ei'u" => Self::Johe,
			"ju'au" => Self::Juhau,
			"ju'ei" => Self::Juhei,
			"ju'u'i" => Self::Juhuhi,
			"kau'ai" | "kau'au" => Self::Kauhai,
			"kau'u" => Self::Kauhu,
			"kei'au" => Self::Keihau,
			"kei'i" => Self::Keihi,
			"ke'ei" => Self::Kehei,
			"ke'ei'a" => Self::Keheiha,
			"ke'e'au" => Self::Kehehau,
			"ke'e'u" => Self::Kehehu,
			"ke'u'i" => Self::Kehuhi,
			"ku'au" => Self::Kuhau,
			"ku'ei" => Self::Kuhei,
			"ku'oi'u" => Self::Kuhoihu,
			"fy'oi" | "ky'oi" => Self::Kyhoi,
			"le'ai" => Self::Lehai,
			"li'au" => Self::Lihau,
			"li'ei" => Self::Lihei,
			"lo'ai" | "sa'ai" => Self::Lohai,
			"fo'ai" | "ko'ai" | "koi'i" | "lo'oi" | "mau'a" | "xau'a" | "xu'u" => Self::Lohoi,
			"lu'ei" => Self::Luhei,
			"mau'au" => Self::Mauhau,
			"mau'e" => Self::Mauhe,
			"mau'o" => Self::Mauho,
			"mei'e" => Self::Meihe,
			"mei'o" => Self::Meiho,
			"me'oi" => Self::Mehoi,
			"mu'oi" => Self::Muhoi,
			"mu'o'u" => Self::Muhohu,
			"nei'ai" => Self::Neihai,
			"noi'a" | "poi'a" | "poi'o'a" | "soi'a" => Self::Noiha,
			"noi'au" | "poi'au" => Self::Noihau,
			"noi'a'u" | "poi'a'u" => Self::Noihahu,
			"noi'i" => Self::Noihi,
			"no'oi" | "po'oi" => Self::Nohoi,
			"rau'oi" => Self::Rauho,
			"re'ai'e" => Self::Rehaihe,
			"sau'u" => Self::Sauhu,
			"sa'au" => Self::Sahau,
			"sa'oi" => Self::Sahoi,
			"sei'au" => Self::Seihau,
			"sei'o" => Self::Seiho,
			"se'e'i" | "te'e'a" | "te'e'i" | "ve'e'a" | "ve'e'i" | "ve'e'u" | "xe'e'i" | "xe'e'o"
			| "xe'e'u" | "ze'e'a" | "ze'e'au" | "ze'e'e" | "ze'e'i" | "ze'e'o" | "ze'e'u" => Self::Sehehi,
			"se'oi'oi" => Self::Sehoihoi,
			"si'i'ei" => Self::Sihihei,
			"si'i'oi" => Self::Sihihoi,
			"si'oi" => Self::Sihoi,
			"soi'i" => Self::Soihi,
			"so'e'ai" => Self::Sohehai,
			"so'oi" => Self::Sohoi,
			"tai'u" => Self::Taihu,
			"tau'o" => Self::Tauho,
			"ta'oi" => Self::Tahoi,
			"ta'u'i" | "ta'u'u" => Self::Tahuhi,
			"tei'u" => Self::Teihu,
			"te'oi'oi" => Self::Tehoihoi,
			"toi'e" => Self::Toihe,
			"toi'o" => Self::Toiho,
			"vau'e'oi" => Self::Vauhehoi,
			"vau'o'oi" => Self::Vauhohoi,
			"vu'oi" => Self::Vuhoi,
			"vy'y" => Self::Vyhy,
			"xau'e'o" => Self::Xauheho,
			"xa'oi'a'oi'a" => Self::Xahoihahoiha,
			"xe'au" => Self::Xehau,
			"no'au" | "xoi" => Self::Xoi,
			"xo'a" => Self::Xoha,
			"xo'e'o'e" => Self::Xohehohe,
			"xo'i" => Self::Xohi,
			"xu'au" => Self::Xuhau,
			"ji'o'e" | "y'i" => Self::Yhi,
			"zai'ai" => Self::Zaihai,
			"rai'o" | "zai'u" => Self::Zaihu,
			"zau'e'u" => Self::Zauhehu,
			"zei'ei" => Self::Zeihei,
			"zei'oi" => Self::Zeihoi,
			"zi'e'a" | "zi'e'e" | "zi'e'i" | "zi'e'o" | "zi'e'u" => Self::Ziheha,
			"zi'e'au" => Self::Zihehau,
			"zei'o" | "zi'oi" => Self::Zihoi,
			"zoi'ai" => Self::Zoihai,
			"zoi'ai'e" | "zoi'o'e" => Self::Zoihohe,
			"zo'au" => Self::Zohau,
			"zo'e'u" => Self::Zohehu,
			"zo'i'o" => Self::Zohiho,
			"la'oi" | "ra'oi" | "zo'oi" => Self::Zohoi,

			// the order here is important and matches that of `crate::rules::lojban_word` and `crate::rules::brivla_minimal`
			other if crate::rules::cmevla(other).is_some() => Self::Cmevla,
			other
				if crate::rules::cmavo_minimal(other)
					.and_peek(crate::rules::eof)
					.is_some() =>
			{
				Self::UnknownCmavo
			}
			other if crate::rules::gismu(other).is_some() => Self::Gismu,
			other if crate::rules::fuhivla(other).is_some() => Self::Fuhivla,
			other if crate::rules::lujvo_minimal(other).is_some() => Self::Lujvo,
			_ => Self::AnyText,
		};

		(
			selmaho,
			selmaho.is_fundamentally_experimental() || is_experimental,
		)
	}
}
