use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher};
use argon2::password_hash::SaltString;
use jsonwebtoken::{encode, decode, Header, Validation, Algorithm, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use std::env;
use rand::Rng;
use axum_extra::extract::TypedHeader;
use axum_extra::headers::authorization::{Authorization, Bearer};

/// Shared application state for Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
}

/// A user record.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Input payload for user login.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Input payload for user registration.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Output payload for user login.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub user: User,
}

/// Claims for JWT token.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

/// A Wubi tutorial lesson record.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Lesson {
    pub id: i32,  // Changed from u32 to i32 to match MySQL INT type
    pub character: String,
    pub code: String,
    pub description: String,
}

/// Input payload for creating a new lesson.
#[derive(Debug, Deserialize)]
pub struct NewLesson {
    pub character: String,
    pub code: String,
    pub description: String,
}

/// Input payload for recording progress.
#[derive(Debug, Deserialize)]
pub struct ProgressUpdate {
    pub user_name: String,
    pub lesson_id: u32,
    pub accuracy: f32,
    pub score: i32,
}

/// Default MySQL connection string for local development.
pub fn default_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "mysql://root:sdsSDG123*^DD@127.0.0.1:3306/wubi".to_string()
    })
}

/// A Wubi practice article record.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Article {
    pub id: i32,  // Changed from u32 to i32 to match MySQL INT type
    pub title: String,
    pub content: String,
    pub difficulty: String, // easy, medium, hard
}

/// Input payload for creating a new article.
#[derive(Debug, Deserialize)]
pub struct NewArticle {
    pub title: String,
    pub content: String,
    pub difficulty: String,
}

/// A Wubi root character record.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WubiRoot {
    pub id: i32,  // Changed from u32 to i32 to match MySQL INT type
    pub character: String,
    pub code: String,
    pub position: String, // position in the keyboard
    pub description: String,
}

/// Input payload for creating a new wubi root.
#[derive(Debug, Deserialize)]
pub struct NewWubiRoot {
    pub character: String,
    pub code: String,
    pub position: String,
    pub description: String,
}

/// Initialize the database schema and insert starter lessons.
pub async fn init_db(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(64) NOT NULL UNIQUE,
            email VARCHAR(255) NOT NULL UNIQUE,
            password_hash VARCHAR(255) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create lessons table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS lessons (
            id INT AUTO_INCREMENT PRIMARY KEY,
            character_val VARCHAR(32) NOT NULL,
            code VARCHAR(32) NOT NULL,
            description TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create user_progress table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_progress (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_name VARCHAR(64) NOT NULL,
            lesson_id INT NOT NULL,
            accuracy FLOAT NOT NULL,
            score INT NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create articles table for practice texts
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS articles (
            id INT AUTO_INCREMENT PRIMARY KEY,
            title VARCHAR(255) NOT NULL,
            content TEXT NOT NULL,
            difficulty ENUM('easy', 'medium', 'hard') DEFAULT 'medium'
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create wubi_characters table for all chinese characters and their wubi codes
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS wubi_characters (
            id INT AUTO_INCREMENT PRIMARY KEY,
            character_val VARCHAR(4) NOT NULL UNIQUE,
            wubi_code VARCHAR(8) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create wubi_roots table for wubi root characters
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS wubi_roots (
            id INT AUTO_INCREMENT PRIMARY KEY,
            character_val VARCHAR(32) NOT NULL,
            code VARCHAR(32) NOT NULL,
            position VARCHAR(64) NOT NULL,
            description TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Insert sample lessons if table is empty
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM lessons").fetch_one(pool).await?;
    if count == 0 {
        let rows = [
            ("人", "WG", "练习“人”字的五笔编码。"),
            ("日", "KH", "练习“日”字的五笔编码。"),
            ("山", "FQ", "练习“山”字的五笔编码。"),
        ];

        for (character, code, description) in rows {
            sqlx::query(
                "INSERT INTO lessons (character_val, code, description) VALUES (?, ?, ?)"
            )
            .bind(character)
            .bind(code)
            .bind(description)
            .execute(pool)
            .await?;
        }
    }

    // Insert sample articles if table is empty
    let count_result: Result<i64, sqlx::Error> = sqlx::query_scalar("SELECT COUNT(*) FROM articles")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            eprintln!("Error querying articles count: {:?}", e);
            e
        });
    
    if let Ok(count) = count_result {
        if count == 0 {
            let rows = [
                ("练习文章一", "五笔字型是一种高效的中文输入法，通过拆分汉字为基本字根进行输入。", "easy"),
                ("练习文章二", "学习五笔需要掌握字根分布和拆字规则，多加练习才能熟练运用。", "medium"),
                ("练习文章三", "汉字的结构复杂多样，五笔输入法按照汉字的笔画和结构规律进行编码。", "hard"),
            ];

            for (title, content, difficulty) in rows {
                if let Err(e) = sqlx::query(
                    "INSERT INTO articles (title, content, difficulty) VALUES (?, ?, ?)"
                )
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .execute(pool)
                .await {
                    eprintln!("Error inserting article: {:?}", e);
                }
            }
        }
    }

    // Insert sample wubi roots if table is empty
    let count_result: Result<i64, sqlx::Error> = sqlx::query_scalar("SELECT COUNT(*) FROM wubi_roots")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            eprintln!("Error querying wubi_roots count: {:?}", e);
            e
        });
    
    if let Ok(count) = count_result {
        if count == 0 {
            let rows = [
                ("一", "GGLL", "G区第一键", "横区起首字根"),
                ("丿", "TTLL", "T区第一键", "撇区起首字根"),
                ("丨", "HHLL", "H区第一键", "竖区起首字根"),
                ("丶", "YYLL", "Y区第一键", "捺区起首字根"),
                ("乙", "NNLL", "N区第一键", "折区起首字根"),
                ("九", "VTNG", "V区第二键", "字根：乙"),
                ("力", "LTNN", "L键", "字根：力"),
                ("乃", "DETN", "N键", "字根：乃"),
                ("刀", "VNTE", "V键", "字根：刀"),
                ("卜", "HHYD", "H键", "字根：卜"),
            ];

            for (character, code, position, description) in rows {
                if let Err(e) = sqlx::query(
                    "INSERT INTO wubi_roots (character_val, code, position, description) VALUES (?, ?, ?, ?)"
                )
                .bind(character)
                .bind(code)
                .bind(position)
                .bind(description)
                .execute(pool)
                .await {
                    eprintln!("Error inserting wubi root: {:?}", e);
                }
            }
        }
    }

    // Clear and repopulate wubi characters table with comprehensive data
    sqlx::query("DELETE FROM wubi_characters")
        .execute(pool)
        .await
        .map_err(|e| {
            eprintln!("Error clearing wubi_characters table: {:?}", e);
            e
        })?;

    let rows = [
        ("王", "gggg"), ("李", "sbvb"), ("张", "xtay"), ("刘", "yjlv"), ("陈", "bii"),
        ("杨", "mnw"), ("赵", "fhq"), ("黄", "amw"), ("周", "mfkf"), ("吴", "kgdu"),
        ("你", "wqiy"), ("好", "qynu"), ("我", "trnt"), ("们", "wwxn"), ("学", "ipbf"),
        ("习", "nyud"), ("五", "gqvg"), ("笔", "thtc"), ("的", "rqyy"), ("一", "ggl"),
        ("是", "jgh"), ("在", "fhg"), ("人", "wwgg"), ("有", "def"), ("中", "khk"),
        ("大", "dddd"), ("为", "ylge"), ("上", "hhll"), ("个", "whjh"), ("国", "lgyi"),
        ("家", "pgte"), ("经", "xcta"), ("可", "skd"), ("以", "nywy"), ("年", "rhfk"),
        ("月", "eeg"), ("日", "jjjj"), ("时", "jfy"), ("会", "wfo"), ("生", "tgj"),
        ("工", "aaaa"), ("都", "ftjb"), ("下", "ghhg"), ("要", "svf"), ("说", "yukq"),
        ("就", "uqiy"), ("出", "bmkg"), ("作", "wtfg"), ("地", "fny"), ("方", "yygy"),
        ("成", "dnnt"), ("市", "ymhj"), ("民", "nav"), ("十", "fgh"), ("公", "awu"),
        ("司", "ngkd"), ("电", "jnwy"), ("话", "xyyy"), ("号", "kgkg"), ("他", "wbnn"),
        ("她", "vbn"), ("它", "pkn"), ("这", "ypt"), ("那", "vfb"), ("里", "jf"),
        ("来", "goi"), ("去", "fcu"), ("多", "qqqu"), ("少", "it"), ("天", "gd"),
        ("空", "pfw"), ("水", "ii"), ("火", "ooo"), ("木", "ssss"), ("金", "qqq"),
        ("土", "ffff"), ("田", "llll"), ("白", "rrr"), ("手", "rtgh"), ("足", "khu"),
        ("心", "nyu"), ("见", "mqb"), ("言", "yyyy"), ("食", "wnfv"), ("衣", "pu"),
        ("门", "uygh"), ("车", "lg"), ("马", "cnnd"), ("牛", "rhk"), ("羊", "ud"),
        ("犬", "dgty"), ("鸟", "qyng"), ("虫", "jhnu"), ("鱼", "qogf"), ("龙", "dxai"),
        ("虎", "haht"), ("兔", "qkqy"), ("猴", "qtwy"), ("鸡", "cqwy"), ("狗", "qtqy"),
        ("猫", "ptqy"), ("猪", "qtyf"), ("鼠", "vnun"), ("蛇", "jxfn"), ("凤", "mcqi"),
        ("凰", "rqrm"), ("龟", "qjnf"), ("虾", "jghy"), ("蟹", "qevg"), ("贝", "mhny"),
        ("蛋", "nguf"), ("肉", "meyi"), ("骨", "meqr"), ("血", "tld"), ("脉", "eryg"),
        ("筋", "erlf"), ("皮", "gqhc"), ("毛", "tfnn"), ("发", "vty"), ("头", "udf"),
        ("脸", "emnk"), ("眼", "hv"), ("睛", "hgqe"), ("眉", "nhty"), ("鼻", "thlj"),
        ("嘴", "khqc"), ("唇", "dfhe"), ("舌", "tdd"), ("牙", "ahen"), ("齿", "hwbn"),
        ("耳", "bgfh"), ("听", "kr"), ("闻", "ubd"), ("声", "fnr"), ("音", "ujf"),
        ("乐", "qi"), ("歌", "sksw"), ("舞", "rlgh"), ("唱", "kjjf"), ("讲", "yfjh"),
        ("谈", "yooy"), ("论", "ywvn"), ("议", "yqrg"), ("评", "gyyh"), ("读", "yfcd"),
        ("写", "pgng"), ("字", "pvfy"), ("词", "yngk"), ("语", "ykgk"), ("文", "yygy"),
        ("章", "ujjh"), ("句", "kbd"), ("段", "wdmc"), ("篇", "thgu"), ("书", "mhf"),
        ("画", "glm"), ("图", "ltk"), ("照", "jhqo"), ("相", "sqkg"), ("片", "thgf"),
        ("影", "jyie"), ("视", "pmm"), ("脑", "epqb"), ("机", "sm"), ("器", "kkdk"),
        ("件", "wrhf"), ("程", "tkgg"), ("技", "rfcy"), ("术", "syty"), ("科", "tu"),
        ("研", "dga"), ("究", "pwvh"), ("实", "pmbj"), ("验", "cgx"), ("室", "pgcf"),
        ("教", "ftbt"), ("育", "yce"), ("培", "fukg"), ("训", "ykhh"), ("老", "ftxb"),
        ("师", "jgmh"), ("校", "suq"), ("院", "bpfi"), ("系", "txiu"), ("班", "gytg"),
        ("级", "xe"), ("课", "yjsy"), ("堂", "ipkf"), ("桌", "hsgg"), ("椅", "dsff"),
        ("黑", "lfo"), ("板", "gcgi"), ("粉", "owvn"), ("纸", "mqyy"), ("墨", "lfof"),
        ("砚", "dmqn"), ("琴", "ggwn"), ("棋", "sdgt"), ("诗", "ytfh"), ("赋", "mgdy"),
        ("武", "gah"), ("剑", "qjyh"), ("刀", "vntr"), ("枪", "swyb"), ("棍", "sjfg"),
        ("棒", "sdjh"), ("球", "gki"), ("拍", "rrgc"), ("网", "mqyy"), ("运", "fcpn"),
        ("动", "fce"), ("比", "xxxi"), ("赛", "pfjm"), ("竞", "ujwf"), ("体", "wety"),
        ("健", "wvf"), ("身", "mhty"), ("跑", "khqn"), ("步", "ihyy"), ("跳", "khij"),
        ("跃", "khtd"), ("飞", "nui"), ("翔", "udng"), ("腾", "eudc"), ("冲", "ikh"),
        ("击", "fmkg"), ("打", "rsyy"), ("搏", "rgef"), ("斗", "ukf"), ("争", "qvyy"),
        ("战", "hkyy"), ("胜", "etgg"), ("利", "tjh"), ("功", "lkln"), ("劳", "apl"),
        ("苦", "adf"), ("甜", "tfqd"), ("酸", "sgct"), ("辣", "ugki"), ("咸", "dgkl"),
        ("美", "ugdu"), ("味", "kfiy"), ("香", "tjf"), ("臭", "tgmk"), ("色", "qcb"),
        ("彩", "gesy"), ("红", "xa"), ("橙", "swgu"), ("绿", "xiii"), ("青", "gef"),
        ("蓝", "ijtl"), ("紫", "egx"), ("灰", "do"), ("褐", "pfwo"), ("粉", "owvn"),
        ("颜", "utem"), ("料", "ouq"), ("染", "ivyy"), ("光", "iqv"), ("明", "je"),
        ("亮", "ypmb"), ("暗", "ju"), ("强", "xk"), ("弱", "xux"), ("软", "lux"),
        ("硬", "mjq"), ("轻", "lcag"), ("重", "tgjf"), ("高", "ymkf"), ("低", "wqa"),
        ("长", "ta"), ("短", "tdgg"), ("宽", "pqdp"), ("窄", "pwfk"), ("厚", "djb"),
        ("薄", "aigf"), ("粗", "oe"), ("细", "xl"), ("小", "ihyy"), ("巨", "and"),
        ("微", "tmg"), ("远", "fqp"), ("近", "wp"), ("左", "da"), ("右", "dkf"),
        ("前", "ue"), ("后", "rg"), ("东", "ai"), ("西", "sgh"), ("南", "fmf"),
        ("北", "ux"), ("央", "md"), ("世", "anvv"), ("界", "lw"), ("宇", "pgf"),
        ("宙", "pmf"), ("星", "jtgf"), ("辰", "dj"), ("昨", "jtfg"), ("今", "wynb"),
        ("晨", "jdfe"), ("晚", "jqk"), ("夜", "pwqc"), ("午", "tfjf"), ("早", "jhnh"),
        ("春", "dwjf"), ("夏", "dfff"), ("秋", "to"), ("冬", "tuf"), ("四", "lh"),
        ("季", "tb"), ("分", "iwv"), ("秒", "tih"), ("刻", "ynt"), ("节", "ab"),
        ("气", "rng"), ("温", "ijlg"), ("度", "ya"), ("冷", "uwyc"), ("热", "rvyo"),
        ("暖", "jefc"), ("寒", "pfj"), ("暑", "jfjf"), ("冰", "ui"), ("雪", "fv"),
        ("霜", "sf"), ("雾", "fqkg"), ("露", "khkf"), ("雨", "fvte"), ("云", "fcu"),
        ("雷", "flf"), ("风", "mqi"), ("沙", "imm"), ("尘", "iff"), ("石", "dgtg"),
        ("山", "mmtt"), ("河", "isk"), ("海", "itx"), ("江", "ia"), ("湖", "ideg"),
        ("波", "ihcy"), ("浪", "iyve"), ("潮", "ifj"), ("汐", "jqn"), ("流", "iyc"),
        ("泉", "riu"), ("溪", "irqy"), ("川", "kthh"), ("泊", "irg"), ("港", "iawn"),
        ("湾", "iom"), ("洋", "iu"), ("洲", "yty"), ("岛", "qmlg"), ("礁", "dgr"),
        ("岩", "mdf"), ("矿", "dqkg"), ("物", "tr"), ("宝", "pgyl"), ("玉", "gyi"),
        ("珠", "grgg"), ("玛", "gqkg"), ("瑙", "grm"), ("珍", "gwet"), ("瑰", "grq"),
        ("银", "qveg"), ("铜", "qmkg"), ("铁", "qrh"), ("钢", "qmq"), ("铝", "qgq"),
        ("锌", "qqfg"), ("锡", "qjq"), ("铅", "qwk"), ("汞", "aqyg"), ("硅", "dqkg"),
        ("碳", "dmy"), ("氢", "rqv"), ("氧", "jrr"), ("氮", "rbn"), ("氯", "rdht"),
        ("氦", "rnfg"), ("氖", "rmn"), ("氩", "rmqn"), ("氪", "rmqn"), ("氙", "rmqn"),
        ("氡", "rmqn"), ("钫", "qynn"), ("镭", "qqyy"), ("锕", "qqwc"), ("钍", "qnnr"),
        ("镤", "qrcy"), ("铀", "qtdg"), ("镎", "qpdg"), ("钚", "qgmi"), ("镅", "qtrc"),
        ("锔", "qtdg"), ("锫", "qtrc"), ("锎", "qtdg"), ("锿", "qtrc"), ("镄", "qtdg"),
        ("钔", "qtrc"), ("锘", "qtrc"), ("铹", "qtrc"), ("垆", "qtrc"), ("杜", "sfg"),
        ("邦", "dtbh"), ("际", "vwfi"), ("组", "xeg"), ("织", "xkfh"), ("委", "tvfg"),
        ("员", "kfmu"), ("坛", "fyhb"), ("社", "pyty"), ("团", "lffe"), ("队", "nwy"),
        ("伍", "wfg"), ("军", "pl"), ("士", "fghg"), ("兵", "rw"), ("官", "pnv"),
        ("领", "wycb"), ("导", "jty"), ("干", "fggh"), ("部", "ukhb"), ("政", "ghty"),
        ("府", "fwfh"), ("法", "ifcy"), ("律", "tvfh"), ("规", "fwm"), ("制", "rmhj"),
        ("管", "tpq"), ("理", "wj"), ("监", "jtyl"), ("督", "hity"), ("检", "swy"),
        ("查", "sjfg"), ("审", "pjfh"), ("计", "tfm"), ("核", "synw"), ("算", "tawu"),
        ("统", "xycy"), ("析", "sr"), ("调", "ymfk"), ("开", "ga"), ("发", "ntcy"),
        ("设", "ywg"), ("进", "fjpk"), ("步", "ihyy"), ("展", "vte"), ("推", "rwyg"),
        ("促", "wkkg"), ("助", "eleg"), ("力", "ltan"), ("帮", "dlg"), ("扶", "rfkg"),
        ("持", "rffg"), ("支", "fcyy"), ("援", "refe"), ("合", "wgkf"), ("协", "flwy"),
        ("配", "sgnp"), ("搭", "rawk"), ("建", "vfkg"), ("造", "tfky"), ("施", "ytb"),
        ("项", "admy"), ("目", "ajkd"), ("划", "sjfh"), ("安", "pvf"), ("排", "rhcb"),
        ("任", "wtfg"), ("务", "teyc"), ("标", "sfiy"), ("的", "rqyy"), ("向", "tmkf"),
        ("路", "khdk"), ("径", "caeg"), ("途", "wkqp"), ("道", "uthp"), ("桥", "sthc"),
        ("梁", "ivws"), ("隧", "kans"), ("轨", "lvcc"), ("公", "awu"), ("速", "kmhk"),
        ("交", "ty"), ("通", "cep"), ("输", "cgop"), ("航", "teyg"), ("票", "trtf"),
        ("登", "tufh"), ("舱", "tefn"), ("位", "wug"), ("舒", "wfcb"), ("适", "udkb"),
        ("旅", "ytey"), ("游", "mqyy"), ("景", "jyiu"), ("点", "hk"), ("名", "qkq"),
        ("古", "dhg"), ("迹", "yndx"), ("历", "dtx"), ("史", "kqvh"), ("化", "wxn"),
        ("传", "wfy"), ("习", "nyud"), ("俗", "wwwy"), ("情", "nge"), ("族", "kmmc"),
        ("承", "wgen"), ("扬", "rnrt"), ("播", "ftgy"), ("媒", "vhkg"), ("新", "usjl"),
        ("闻", "ubd"), ("报", "rcbh"), ("刊", "tqjh"), ("杂", "vsyy"), ("志", "qvh"),
        ("籍", "tfo"), ("馆", "qknn"), ("档", "sfgn"), ("案", "sfi"), ("资", "uqwm"),
        ("信", "wygg"), ("息", "wht"), ("数", "ov"), ("据", "rwyk"), ("络", "xlq"),
        ("互", "gxfg"), ("联", "bwyc"), ("站", "hhkg"), ("页", "kkhk"), ("面", "dlm"),
        ("功", "lkln"), ("能", "dmnu"), ("菜", "aes"), ("单", "sfkw"), ("选", "tfq"),
        ("择", "nwyh"), ("确", "hpq"), ("认", "ybg"), ("否", "gik"), ("定", "pgjh"),
        ("赞", "deu"), ("同", "mgkd"), ("反", "rci"), ("对", "cfyh"), ("拥", "rtdy"),
        ("护", "rynt"), ("爱", "epd"), ("关", "udvf"), ("善", "uduk"), ("和", "t"),
        ("平", "gufh"), ("结", "kjff"), ("共", "a"), ("努", "vcfy"), ("奋", "dhah"),
        ("拼", "rkuj"), ("赢", "fcl"), ("绩", "tm"), ("等", "tffu"), ("次", "uqwu"),
        ("排", "rhcb"), ("第", "txht"), ("二", "fgg"), ("三", "dggg"), ("六", "ynty"),
        ("七", "agq"), ("八", "wty"), ("九", "vtng"), ("百", "djf"), ("千", "tfk"),
        ("万", "dnv"), ("亿", "wnxn"), ("零", "fwyc"), ("壹", "fpgg"), ("贰", "afmg"),
        ("叁", "ddf"), ("肆", "dvg"), ("伍", "fgty"), ("陆", "bff"), ("柒", "asa"),
        ("捌", "rty"), ("玖", "gqvg"), ("拾", "rwfh"), ("内", "mwu"), ("外", "qhng"),
        ("边", "lpv"), ("旁", "upyh"), ("侧", "wmjy"), ("置", "tfhy"), ("址", "fhgy"),
        ("区", "aqyy"), ("域", "akgb"), ("范", "aib"), ("围", "lfnh"), ("畴", "aibh"),
        ("概", "sjvc"), ("念", "pyjn"), ("意", "ujnu"), ("义", "yqwt"), ("含", "wgku"),
        ("解", "qevy"), ("释", "toc"), ("阐", "yjfg"), ("述", "syy"), ("表", "gteg"),
        ("达", "udpp"), ("示", "lmqi"), ("显", "jo"), ("呈", "kgjk"), ("现", "qmd"),
        ("演", "uje"), ("练", "unwl"), ("践", "prhh"), ("操", "rkr"), ("使", "wgk"),
        ("应", "yid"), ("具", "dhnn"), ("材", "sff"), ("备", "tmwy"), ("装", "ufye"),
        ("械", "mad"), ("子", "bjf"), ("产", "ute"), ("品", "gqtf"), ("商", "umgd"),
        ("质", "rfq"), ("量", "wfg"), ("标", "sfiy"), ("准", "uwy"), ("格", "stk"),
        ("型", "fgf"), ("尺", "bbfy"), ("寸", "fghy"), ("容", "pww"), ("积", "tkhy"),
        ("密", "pnt"), ("角", "qej"), ("率", "yxnr"), ("因", "qxv"), ("果", "jtgy"),
        ("原", "dr"), ("理", "djgg"), ("则", "mjk"), ("条", "ts"), ("约", "xqyy"),
        ("需", "fqwy"), ("必", "qntt"), ("须", "wqvh"), ("当", "ivfg"), ("能", "cej"),
        ("够", "qksk"), ("性", "shgy"), ("素", "gxi"), ("键", "nvfh"), ("主", "yhgd"),
        ("般", "cen"), ("常", "ipkh"), ("普", "ogy"), ("遍", "xxgf"), ("广", "yygt"),
        ("泛", "itcy"), ("殊", "wqwy"), ("专", "fny"), ("业", "vhlg"), ("职", "bkhy"),
        ("责", "efcy"), ("权", "scyu"), ("巧", "agnn"), ("段", "wdmc"), ("措", "ryt"),
        ("策", "tgn"), ("略", "yjty"), ("想", "shnu"), ("思", "lnu"), ("考", "ftxi"),
        ("探", "rpwy"), ("索", "rpys"), ("现", "qmqy"), ("明", "jlo"), ("创", "wjb"),
        ("改", "nty"), ("完", "pfqb"), ("善", "uhdu"), ("优", "wxd"), ("提", "ryk"),
        ("升", "hxf"), ("增", "fxjh"), ("降", "bc"), ("减", "uwgi"), ("控", "yfcb"),
        ("评", "gyyh"), ("估", "wwyg"), ("测", "imjh"), ("试", "yaff"), ("证", "ygkg"),
        ("保", "wks"), ("维", "xwyg"), ("养", "udhl"), ("修", "wht"), ("复", "tut"),
        ("恢", "niq"), ("还", "pl"), ("状", "utby"), ("态", "sdg"), ("况", "hk"),
        ("趋", "fhwy"), ("势", "ukhy"), ("执", "rvfy"), ("行", "tfrh"), ("落", "ait"),
        ("到", "gctp"), ("转", "lfn"), ("换", "qpl"), ("变", "yrc"), ("整", "tfjq"),
        ("兼", "uvq"), ("稳", "tqv"), ("靠", "trqn"), ("全", "wqj"), ("效", "wkh"),
        ("约", "xqyy"), ("环", "gkg"), ("续", "wht"), ("济", "xyj"), ("场", "fnrt"),
        ("贸", "qyvd"), ("易", "sjfg"), ("买", "nudu"), ("卖", "pfny"), ("购", "mqcy"),
        ("销", "mjgf"), ("售", "sfkg"), ("批", "rxhc"), ("价", "wwjh"), ("本", "sgm"),
        ("润", "iann"), ("收", "nwe"), ("入", "wqvy"), ("盈", "eudv"), ("亏", "fnwy"),
        ("损", "rwyk"), ("投", "rmcy"), ("融", "jtfq"), ("贷", "wayg"), ("款", "wvkg"),
        ("储", "wyfy"), ("蓄", "dejj"), ("存", "dhwn"), ("用", "weyg"), ("卡", "hnn"),
        ("债", "wbnt"), ("券", "gvqn"), ("股", "emcy"), ("期", "dwe"), ("货", "trqc"),
        ("汇", "xng"), ("险", "wgu"), ("财", "mxwj"), ("税", "ftb"), ("企", "thdu"),
        ("司", "ngkd"), ("单", "ujhf"), ("构", "swk"), ("薪", "uefh"), ("酬", "sdff"),
        ("资", "fjqr"), ("奖", "uqvu"), ("金", "qwri"), ("福", "pygl"), ("利", "txjh"),
        ("障", "pylg"), ("医", "atd"), ("疗", "qkgn"), ("康", "yvi"), ("讯", "ykkg"),
        ("源", "elf"), ("境", "fjn"), ("农", "peyi"), ("林", "sswy"), ("牧", "try"),
        ("渔", "iqgg"), ("服", "ebh"), ("移", "ttqq"), ("智", "tdkj"), ("云", "fcu"),
        ("块", "frqc"), ("链", "gkpf"), ("虚", "hao"), ("拟", "xnu"), ("增", "fxjh"),
        ("机", "wsm"), ("互", "gxfg"), ("户", "wtyg"), ("视", "sgx"), ("布", "dmhc"),
        ("式", "khjc"), ("并", "gcag"), ("行", "tfrh"), ("处", "thcy"), ("构", "sfkg"),
        ("协", "flwy"), ("加", "lka"), ("防", "byn"), ("墙", "ffg"), ("杀", "qsyy"),
        ("毒", "gmy"), ("件", "ujfh"), ("架", "fjfg"), ("测", "imjh"), ("署", "ukht"),
        ("维", "xwyg"), ("故", "dty"), ("障", "ufvh"), ("除", "bh"), ("性", "shgy"),
        ("试", "yaff"), ("码", "llcy"), ("档", "sfgn"), ("版", "thgc"), ("沟", "ixcy"),
        ("流", "iyc"), ("享", "yiyf"), ("精", "oge"), ("神", "pyju"), ("敬", "aqkt"),
        ("群", "vtkd"), ("诚", "yif"), ("负", "zmu"), ("献", "dwfh"), ("愿", "qere"),
        ("者", "ftfg"), ("活", "ftxi"), ("参", "cdyy"), ("与", "gkfg"), ("注", "kkg"),
        ("困", "lsxi"), ("难", "uvgh"), ("童", "ujgg"), ("残", "gqaj"), ("疾", "nxan"),
        ("妇", "vvfg"), ("女", "vvvv"), ("毕", "xgif"), ("求", "fiwy"), ("创", "wjb"),
        ("消", "iqyy"), ("费", "mqyy"), ("客", "pkt"), ("供", "wfi"), ("伙", "wsyy"),
        ("伴", "wufh"), ("盟", "alf"), ("竞", "ujwf"), ("手", "rtgh"), ("营", "aplb"),
        ("销", "pjdh"), ("牌", "gkhg"), ("形", "gaet"), ("象", "qje"), ("宣", "pgjg"),
        ("传", "wfy"), ("告", "yvkg"), ("渠", "ikh"), ("代", "way"), ("店", "sgkf"),
        ("城", "dwne"), ("台", "ckf"), ("付", "wfhy"), ("钱", "qkqn"), ("包", "qnv"),
        ("扫", "rpvy"), ("码", "llcy"), ("期", "wqay"), ("免", "qkqn"), ("费", "mxjf"),
        ("退", "vep"), ("线", "xft"), ("馈", "xwqn"), ("满", "agwn"), ("诉", "kwcy"),
        ("议", "yvre"), ("洁", "sdg"), ("观", "mqqn"), ("易", "jqr"), ("便", "wgjh"),
        ("捷", "wgjh"), ("快", "nnwy"), ("隐", "bqvh"), ("私", "vihn"), ("息", "wht"),
        ("份", "wfh"), ("灾", "soo"), ("动", "fcpf"), ("同", "mgkd"), ("步", "hiho"),
        ("办", "wgjh"), ("程", "tkgg"), ("信", "wygg"), ("即", "vca"), ("聊", "bqt"),
        ("博", "flgy"), ("众", "wwcy"), ("短", "tdgg"), ("邮", "ebcy"), ("箱", "segg"),
        ("送", "uceh"), ("知", "tkwy"), ("醒", "yjt"), ("历", "qqj"), ("闹", "ybg"),
        ("钟", "qkhh"), ("踪", "khgp"), ("控", "swac"), ("告", "rgkg"), ("挖", "fpqn"),
        ("掘", "fqvh"), ("屏", "dsk"), ("茂", "adcn"), ("丰", "dhkf"), ("富", "pgkf"),
        ("彩", "gesy"), ("泼", "igkg"), ("趣", "fwff"), ("吸", "evcy"), ("引", "xhny"),
        ("颖", "jqdh"), ("独", "qtuy"), ("特", "trtf"), ("个", "whjh"), ("制", "rmhj"),
        ("验", "tgqn"), ("优", "wxd"), ("界", "lw"),
    ];

    for (character, code) in rows {
        if let Err(e) = sqlx::query(
            "INSERT INTO wubi_characters (character_val, wubi_code) VALUES (?, ?)"
        )
        .bind(character)
        .bind(code)
        .execute(pool)
        .await {
            eprintln!("Error inserting wubi character: {:?}", e);
        }
    }

    Ok(())
}

/// A simple health check endpoint.
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

/// GET /api/lessons
pub async fn get_lessons(State(state): State<AppState>) -> Result<Json<Vec<Lesson>>, StatusCode> {
    let lessons = sqlx::query_as::<_, Lesson>(
        "SELECT id, character_val AS `character`, code, description FROM lessons ORDER BY id",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(lessons))
}

/// GET /api/lessons/{id}
pub async fn get_lesson(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<Lesson>, StatusCode> {
    let lesson = sqlx::query_as::<_, Lesson>(
        "SELECT id, character_val AS `character`, code, description FROM lessons WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Json(lesson))
}

/// POST /api/lessons
pub async fn create_lesson(
    State(state): State<AppState>,
    Json(payload): Json<NewLesson>,
) -> Result<(StatusCode, Json<Lesson>), (StatusCode, String)> {
    let result = sqlx::query(
        "INSERT INTO lessons (character_val, code, description) VALUES (?, ?, ?)"
    )
    .bind(&payload.character)
    .bind(&payload.code)
    .bind(&payload.description)
    .execute(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let id = result.last_insert_id();
    let lesson = sqlx::query_as::<_, Lesson>(
        "SELECT id, character_val AS `character`, code, description FROM lessons WHERE id = ?",
    )
    .bind(id as u32)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok((StatusCode::CREATED, Json(lesson)))
}

/// POST /api/progress
pub async fn post_progress(
    State(state): State<AppState>,
    Json(payload): Json<ProgressUpdate>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query(
        "INSERT INTO user_progress (user_name, lesson_id, accuracy, score) VALUES (?, ?, ?, ?)"
    )
    .bind(&payload.user_name)
    .bind(payload.lesson_id)
    .bind(payload.accuracy)
    .bind(payload.score)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

/// GET /api/articles
pub async fn get_articles(State(state): State<AppState>) -> Result<Json<Vec<Article>>, StatusCode> {
    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, content, difficulty FROM articles ORDER BY id",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(articles))
}

/// GET /api/articles/{id}
pub async fn get_article(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<Article>, StatusCode> {
    let article = sqlx::query_as::<_, Article>(
        "SELECT id, title, content, difficulty FROM articles WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Json(article))
}

/// POST /api/articles
pub async fn create_article(
    State(state): State<AppState>,
    Json(payload): Json<NewArticle>,
) -> Result<(StatusCode, Json<Article>), (StatusCode, String)> {
    let result = sqlx::query(
        "INSERT INTO articles (title, content, difficulty) VALUES (?, ?, ?)"
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&payload.difficulty)
    .execute(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let id = result.last_insert_id();
    let article = sqlx::query_as::<_, Article>(
        "SELECT id, title, content, difficulty FROM articles WHERE id = ?",
    )
    .bind(id as u32)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok((StatusCode::CREATED, Json(article)))
}

/// GET /api/wubi-roots
pub async fn get_wubi_roots(State(state): State<AppState>) -> Result<Json<Vec<WubiRoot>>, StatusCode> {
    match sqlx::query_as::<_, WubiRoot>(
        "SELECT id, character_val AS `character`, code, position, description FROM wubi_roots ORDER BY position",
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(roots) => Ok(Json(roots)),
        Err(e) => {
            eprintln!("Error fetching wubi roots: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /api/wubi-roots/{id}
pub async fn get_wubi_root(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<WubiRoot>, StatusCode> {
    let root = sqlx::query_as::<_, WubiRoot>(
        "SELECT id, character_val AS `character`, code, position, description FROM wubi_roots WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Json(root))
}

/// POST /api/wubi-roots
pub async fn create_wubi_root(
    State(state): State<AppState>,
    Json(payload): Json<NewWubiRoot>,
) -> Result<(StatusCode, Json<WubiRoot>), (StatusCode, String)> {
    let result = sqlx::query(
        "INSERT INTO wubi_roots (character_val, code, position, description) VALUES (?, ?, ?, ?)"
    )
    .bind(&payload.character)
    .bind(&payload.code)
    .bind(&payload.position)
    .bind(&payload.description)
    .execute(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let id = result.last_insert_id();
    let root = sqlx::query_as::<_, WubiRoot>(
        "SELECT id, character_val AS `character`, code, position, description FROM wubi_roots WHERE id = ?",
    )
    .bind(id as u32)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok((StatusCode::CREATED, Json(root)))
}

/// Search for wubi root by character
pub async fn search_wubi_root(
    Path(character): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Option<WubiRoot>>, StatusCode> {
    let root = sqlx::query_as::<_, WubiRoot>(
        "SELECT id, character_val AS `character`, code, position, description FROM wubi_roots WHERE character_val = ?",
    )
    .bind(character)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(root))
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct WubiCharacter {
    pub character: String,
    pub wubi_code: String,
}

pub async fn get_wubi_code(
    Path(character): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<WubiCharacter>, StatusCode> {
    if character.chars().count() != 1 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 从数据库查询五笔码
    let result = sqlx::query_as::<_, WubiCharacter>(
        "SELECT character_val AS `character`, wubi_code FROM wubi_characters WHERE character_val = ?"
    )
    .bind(&character)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Json(result))
}

/// Generate JWT token for user.
pub fn generate_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let expiration = Utc::now() + Duration::hours(24);
    
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.timestamp(),
    };
    
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&Header::default(), &claims, &key)
}

/// Validate JWT token.
pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let validation = Validation::new(Algorithm::HS256);
    
    let key = DecodingKey::from_secret(secret.as_bytes());
    let decoded = decode::<Claims>(token, &key, &validation)?;
    Ok(decoded.claims)
}

/// Login user.
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Find user by username
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match user {
        Some(user) => {
            // Verify password
            let argon2 = Argon2::default();
            let password_hash = PasswordHash::new(&user.password_hash)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            if argon2.verify_password(payload.password.as_bytes(), &password_hash)
                .is_err() {
                return Err(StatusCode::UNAUTHORIZED);
            }
            
            // Generate token
            let token = generate_token(user.id)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            Ok(Json(LoginResponse {
                access_token: token,
                user,
            }))
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Logout user (client-side logout by removing token).
pub async fn logout() -> StatusCode {
    // Logout is handled client-side by removing the token
    StatusCode::OK
}

/// Register user.
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Check if username already exists
    let existing_user = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT id FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if existing_user.is_some() {
        return Err(StatusCode::CONFLICT);
    }
    
    // Hash password
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();
    
    // Insert user into database
    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)"
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let user_id = result.last_insert_id() as i32;
    
    // Get the created user
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Generate token
    let token = generate_token(user.id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(LoginResponse {
        access_token: token,
        user,
    }))
}

/// Auth middleware to protect routes.
pub async fn auth_middleware(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let token = auth.token();
    
    // Validate token
    let claims = validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let user_id = claims.sub.parse::<i32>()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Verify user exists
    let user_exists = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT id FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match user_exists {
        Some(_) => {
            // Add user_id to request extensions
            let mut req = req;
            req.extensions_mut().insert(user_id);
            Ok(next.run(req).await)
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}