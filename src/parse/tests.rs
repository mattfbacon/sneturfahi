const CLL_EXAMPLES: &'static [&'static str] = &[
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
	// CLL example says "la an,iis" which is now invalid ca'i byfy
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
];

#[test]
fn cll_examples() {
	for example in CLL_EXAMPLES {
		// this output won't show unless the test fails, in which case the last line will helpfully indicate which test failed
		eprintln!("parsing {example:?}");
		crate::parse(&crate::lex(example).collect::<Result<Vec<_>, _>>().unwrap()).unwrap();
	}
}
