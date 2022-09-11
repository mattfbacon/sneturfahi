macro_rules! assert_parse {
	($($sentence:expr),* $(,)?) => {
		#[test]
		fn assert_parse() {
			$({
				let sentence = $sentence;
				eprintln!(".i caku jai cipra lodu'u gendra fa lu {sentence:?} li'u");
				crate::parse(&crate::lex(sentence).collect::<Result<Vec<_>, _>>().expect("lexing failed")).expect("parsing failed");
			})*
		}
	}
}

assert_parse![
	// 5.1
	"do mamta mi",
	"do patfu mi",
	"ta bloti",
	"ta brablo",
	"ta blotrskunri",
	// 5.2
	"tu pelnimre tricu",
	"la djan barda nanla",
	"mi sutra bajra",
	"mi sutra",
	"ta klama jubme",
	"do barda prenu",
	"do cmalu prenu",
	// 5.3
	"ta cmalu nixli bo ckule",
	"ta cmalu bo nixli ckule",
	"ta cmalu nixli ckule",
	"ta klama bo jubme",
	// 5.4
	"do mutce bo barda gerku bo kavbu",
	"ta melbi cmalu nixli ckule",
	"ta melbi cmalu nixli bo ckule",
	"ta melbi cmalu bo nixli ckule",
	"ta melbi cmalu bo nixli bo ckule",
	"ta cmalu bo nixli bo ckule",
	// 5.5
	"ta ke melbi cmalu ke'e nixli ckule",
	"ta ke ke melbi cmalu ke'e nixli ke'e ckule",
	"ta ke ke melbi cmalu ke'e nixli ke'e ckule",
	"ta ke ke ke melbi cmalu ke'e nixli ke'e ckule ke'e",
	"ta melbi ke cmalu nixli ke'e ckule",
	"ta melbi cmalu ke nixli ckule",
	"ta melbi cmalu ke nixli ckule ke'e",
	"ta melbi ke cmalu nixli ckule",
	"ta melbi ke cmalu nixli ckule ke'e",
	"ta melbi ke cmalu ke nixli ckule",
	"ta melbi ke cmalu ke nixli ckule ke'e",
	"ta melbi ke cmalu ke nixli ckule ke'e ke'e",
	"ta melbi ke cmalu nixli bo ckule",
	"ta melbi ke cmalu nixli bo ckule ke'e",
	// 5.6
	"barda xunre gerku",
	"barda xunre bo gerku",
	"barda je xunre gerku",
	"xunre je barda gerku",
	"barda je pelxu bo xunre gerku",
	"barda je ke pelxu xunre ke'e gerku",
	"barda je pelxu xunre gerku",
	"ta blanu je zdani",
	"ta melbi je nixli ckule",
	"ta ke melbi ckule ke'e je ke nixli ckule",
	"ta ke melbi ckule ke'e je ke nixli ckule ke'e",
	"le bajra cu jinga ja te jinga",
	"blanu naja lenku skapi",
	"xamgu jo tordu nuntavla",
	"vajni ju pluka nuntavla",
	"ricfu je blanu jabo crinu",
	"ricfu je blanu jabo crino bo blanu",
	"ricfu je ke blanu ja crino",
	"ricfu je ke blanu ja crino ke'e",
	"ti blanu joi xunre bolci",
	"ti blanu xunre bolci",
	"ti blanu je xunre bolci",
	"gu'e barda gi xunre gerku",
	"gu'e barda je xunre gi gerku ja mlatu",
	// 5.7
	"ti xamgu zdani",
	"ti xamgu be do bei mi zdani",
	"ti xamgu be do bei mi be'o zdani",
	"ti cmalu be le ka se canlu bei lo'e ckule be'o nixli be le nanca be li mu be'o bei lo merku be'o bo ckule la bryklyn loi pemci le mela nu,IORK prenu le jecta",
	"mi klama be le zarci bei le zdani",
	"mi klama be le zarci bei le zdani be'o",
	"mi klama le zarci le zdani",
	"melbi je cmalu nixli bo ckule",
	"ti xamgu be fi mi bei fe do zdani",
	"ti xamgu be fi mi bei fe do be'o zdani",
	"ti xamgu be fi mi zdani",
	"ti xamgu be fi mi be'o zdani",
	"ta blanu be ga'a mi zdani",
	"ta blanu be ga'a mi be'o zdani",
	"ta blanu zdani ga'a mi",
	"le xamgu be do noi barda cu zdani",
	"le xamgu be do be'o noi barda cu zdani",
	"le xamgu be le ctuca be'o zdani",
	"le xamgu be le ctuca ku be'o zdani",
	// 5.8
	"ta blanu zdani",
	"ta zdani co blanu",
	"mi klama be le zarci bei le zdani be'o troci",
	"mi troci co klama le zarci le zdani",
	"ta nixli ckule co cmalu",
	"ta nixli bo ckule co cmalu",
	"ta cmalu ke nixli ckule co melbi",
	"ta cmalu ke nixli ckule ke'e co melbi",
	"ckule co melbi nixli",
	"ke melbi nixli ke'e ckule",
	"ckule co nixli co cmalu",
	"ke ke cmalu ke'e nixli ke'e ckule",
	"cmalu nixli ckule",
	"mi klama co sutra",
	"mi klama be le zarci be'o co sutra",
	// 5.9
	"la djan klama le zarci",
	"la djan go'i troci",
	"la djan klama be le zarci be'o traci",
	"li vo nu'a su'i li re li re",
	"mi jimpe tu'a loi nu'a su'i nabmi",
	"la prim palvr pamoi cusku",
	// corrected to remove glides
	"la anis joi la asun bruna remei",
	"ti nu zdile kei kumfa",
	"ti zdile kumfa",
	// 5.10
	"le ci nolraitru",
	"la BALtazar cu me le ci nolraitru",
	"do du la djan",
	"do me la djan",
	"ta me lai kraislr karce",
	"ta me lai kraislr me'u karce",
	"re me le ci nolraitru e la djan cu blabi",
	"re me le ci nolraitru e la djan me'u cu blabi",
	"re me le ci nolraitru me'u e la djan cu blabi",
	"ta me la'e le se cusku be do me'u cukta",
	"le me le ci noltraitru me'u nunsalci",
	"le me le ci noltraitru ku me'u nunsalci",
	// 5.11
	"mi prami do",
	"do se prami mi",
	"la alis cu cadzu klama le zarci",
	"le zarci cu se ke cadzu klama ke'e la alis",
	"le zarci cu se cadzu klama la alis",
	"le zarci cu cadzu se klama la alis",
	"la djan cu cadzu se klama la alis",
	// 5.12
	"la alis cu na'e ke cadzu klama le zarci",
	"la alis cu na'e ke cadzu klama ke'e le zarci",
	"la alis cu na'e cadzu klama le zarci",
	"la djonz cu na'e pamoi cusku",
	"mi na'e sutra bo cadzu be fi le birka be'o klama le zarci",
	"mi na'e ke sutra cadzu be fi le birka ke'e klama le zarci",
	"mi na'e ke sutra cadzu be fi le birka be'o ke'e klama le zarci",
	"mi sutra bo cadzu be fi le birka be'o je masno klama le zarci",
	"mi ke sutra cadzu be fi le birka ke'e je masno klama le zarci",
	"mi ke sutra cadzu be fi le birka be'o ke'e je masno klama le zarci",
	"mi na'e ke sutra bo cadzu be fi le birka be'o je masno klama le zarci",
	"mi na'e ke sutra bo cadzu be fi le birka be'o je masno klama ke'e le zarci",
	"mi na'e ke sutra bo cadzu be fi le birka je masno klama le zarci",
	"mi na'e ke sutra bo cadzu be fi le birka je masno klama be'o le zarci",
	"mi na'e ke sutra bo cadzu be fi le birka je masno klama ke'e le zarci",
	"mi na'e ke sutra bo cadzu be fi le birka je masno klama be'o ke'e le zarci",
	// 5.13
	"mi pu klama le zarci",
	"la djonz na pamoi cusku",
	"mi na pu klama le zarci",
	"mi na na klama le zarci",
	"mi na pu na ca klama le zarci",
	// 6.1
	"mi klama le zarci",
	"e'osai ko sarji la lojban",
	"mi cusku lu e'osai li'u le tcidu",
	"ti mitre li ci",
	// 6.2
	"le zarci",
	"le zarci cu barda",
	"le nanmu cu ninmu",
	"lo zarci",
	"lo nanmu cu ninmu",
	"la cribe pu finti le lisri",
	"la stace pu citka lo cirla",
	"lo cribe pu finti le lisri",
	"le remna pu finti le lisri",
	"lo remna pu finti le lisri",
	// 6.3
	"le prenu cu bevri le pipno",
	"lei prenu cu bevri le pipno",
	"loi cinfo cu xabju le fi'ortu'a",
	"loi glipre cu xabju le fi'ortu'a",
	"loi matne cu ranti",
	"lai cribe pu finti le vi cukta",
	// 6.4
	"lo ratcu cu bunre",
	"loi ratcu cu cmalu",
	"lo'i ratcu cu barda",
	"mi fadni zo'e lo'i lobypli",
	// 6.5
	"lo'e cinfo cu xabju le fi'ortu'a",
	"lo'e glipre cu xabju le fi'ortu'a na.e le gligugde",
	"le'e xelso merko cu gusta ponse",
	"le'e skina cu se finti ne'i la xali,uyd",
	// 6.6
	"do cadzu le bisli",
	"re do cadzu le bisli",
	"mi ponse su'o ci cutci",
	"ro do cadzu le bisli",
	"mi cusku lu do cadzu le bisli li'u",
	"mi cusku ro lu do cadzu le bisli li'u",
	"mi cusku su'o lu do cadzu le bisli li'u",
	"mi cusku re lu do cadzu le bisli li'u",
	"re le gerku cu blabi",
	"re le ci gerku cu blabi",
	"le ci gerku cu blabi",
	"ro le ci gerku cu blabi",
	"ci lo gerku cu blabi",
	"ci lo ro gerku cu blabi",
	"so'o lo ci gerku cu blabi",
	// 6.8
	"ci gerku cu blabi",
	"ci gerku ku cu blabi",
	"mi ponse su'o ci lo cutci",
	// 6.9
	"re do cu nanmu",
	"le re do cu nanmu",
	"re le ci cribe cu bunre",
	"le re le ci cribe cu bunre",
	"pa le re le ci cribe cu bunre",
	// 6.10
	"mi viska lu le xunre cmaxirma li'u",
	"mi viska le selsinxa be lu le xunre cmaxirma li'u",
	"mi viska la'e lu le xunre cmaxirma li'u",
	"mi viska la'e lu le xunre cmaxirma li'u lu'u",
	"mi pu cusku lu'e le vi cukta",
	"mi pu cusku le sinxa be le vi cukta",
	"mi troci tu'a le vorme",
	"lo'i ratcu cu barda .iku'i lu'a ri cmalu",
	"lo ratcu cu cmalu .i ku'i lu'i ri barda",
	"mi ce do girzu .i lu'o ri gunma .i vu'i ri porsi",
	"mi viska na'e bo le gerku",
	"mi nelci loi glare cidja .ije do nelci to'ebo ri .ije la djein nelci no'ebo ra",
	// 6.11
	"coi",
	"je'e",
	"coi djan",
	"doi djan",
	"coi xunre pastu nixli",
	"co'o la bab .e la noras",
	"coi le xunre pastu nixli",
	"doi la djan",
	"doi djan ko klama mi",
	"ko klama mi doi djan",
	// 6.12
	"la djonz klama le zarci",
	"lai djonz klama le zarci",
	// corrected to remove glides
	"doi djan pol djonz le bloti cu klama fi la niiuport niiuz",
	// 6.13
	"mi prami do",
	"le cribe goi ko'a cu xekri .i ko'a citka le smacu",
	"ro da poi prenu cu prami pa de poi finpe",
	"le cribe cu batci vo'a",
	"mi klama la frankfurt ri",
	"mi klama la frankfurt zo'e zo'e zo'e",
	"ko muvgau ti ta tu",
	"li re su'i re du li vo .i la'e di'u jetnu",
	"mi viska le mlatu ku poi zo'e zbasu ke'a loi slasi",
	"do klama ma",
	// added to test "In addition, sequences of lerfu words (of selma'o BY and related selma'o) can also be used as definable pro-sumti."
	"bycy broda dypa",
	// 6.14
	"mi cusku lu mi'e djan li'u",
	"mi cusku lo'u li mi le'u",
	"mi cusku zo ai",
	"mi cusku zoi kuot I'm John kuot",
	// 6.15
	"li vo",
	"li re su'i re",
	"li abu bi'epi'i xy bi'ete'a re su'i by bi'epi'i xy su'i cy",
	"me'o vo",
	"me'o re su'i re",
];
