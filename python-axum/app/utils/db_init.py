from sqlalchemy.orm import Session
from app.core.database import engine, Base, SessionLocal
from app.models.models import KeyRadical, EnglishText


def init_db():
    Base.metadata.create_all(bind=engine)


def init_key_radicals(db: Session):
    existing = db.query(KeyRadical).count()
    if existing > 0:
        return

    key_radicals = [
        KeyRadical(
            key_char="g",
            radicals="王、一、五、戋",
            formula="王旁青头戋（兼）五一",
            description="G区横区第一键，包含横笔和戈字根"
        ),
        KeyRadical(
            key_char="f",
            radicals="土、士、二、干，十、寸、雨",
            formula="土士二干十寸雨",
            description="F区横区第二键，包含土字根"
        ),
        KeyRadical(
            key_char="d",
            radicals="大、犬、三、古、石、厂",
            formula="大犬三（古）石厂",
            description="D区横区第三键，包含大字根"
        ),
        KeyRadical(
            key_char="s",
            radicals="木、丁、西",
            formula="木丁西",
            description="S区横区第四键，包含木字根"
        ),
        KeyRadical(
            key_char="a",
            radicals="工、戈、艹、七、廿",
            formula="工戈草头右框七",
            description="A区横区第五键，包含工字根"
        ),
        KeyRadical(
            key_char="h",
            radicals="目、止、卜、虍、上",
            formula="目具上止卜虎皮",
            description="H区竖区第一键，包含目字根"
        ),
        KeyRadical(
            key_char="j",
            radicals="日、早、虫、刂、竖",
            formula="日早两竖与虫依",
            description="J区竖区第二键，包含日字根"
        ),
        KeyRadical(
            key_char="k",
            radicals="口、川",
            formula="口与川，字根稀",
            description="K区竖区第三键，包含口字根"
        ),
        KeyRadical(
            key_char="l",
            radicals="田、甲、四、车、囗",
            formula="田甲方框四车里",
            description="L区竖区第四键，包含田字根"
        ),
        KeyRadical(
            key_char="m",
            radicals="山、由、贝、几",
            formula="山由贝，下框几",
            description="M区竖区第五键，包含山字根"
        ),
        KeyRadical(
            key_char="t",
            radicals="禾、竹、丿、彳、攵",
            formula="禾竹一撇双人立",
            description="T区撇区第一键，包含禾字根"
        ),
        KeyRadical(
            key_char="r",
            radicals="白、手、斤、牛",
            formula="白手看头三二斤",
            description="R区撇区第二键，包含白字根"
        ),
        KeyRadical(
            key_char="e",
            radicals="舟、用、月、豕、衣",
            formula="舟用乃月豕（家）衣",
            description="E区撇区第三键，包含月字根"
        ),
        KeyRadical(
            key_char="w",
            radicals="人、八、亻",
            formula="人八登头单人几",
            description="W区撇区第四键，包含人字根"
        ),
        KeyRadical(
            key_char="q",
            radicals="金、饣、勹、儿、夕",
            formula="金勺缺点无尾鱼，犬旁留叉",
            description="Q区撇区第五键，包含金字根"
        ),
        KeyRadical(
            key_char="y",
            radicals="言、文、方、广、丶",
            formula="言文方广在四一，高头一捺谁人去",
            description="Y区捺区第一键，包含言字根"
        ),
        KeyRadical(
            key_char="u",
            radicals="立、辛、六、门、疒",
            formula="立辛两点六门疒（病）",
            description="U区捺区第二键，包含立字根"
        ),
        KeyRadical(
            key_char="i",
            radicals="氵（三点水）、小",
            formula="水旁兴头小倒立",
            description="I区捺区第三键，包含水字根"
        ),
        KeyRadical(
            key_char="o",
            radicals="火、米、灬",
            formula="火业头，四点米",
            description="O区捺区第四键，包含火字根"
        ),
        KeyRadical(
            key_char="p",
            radicals="之、宀（宝盖）、冖、礻、衤",
            formula="之字军盖建道底，摘礻衤",
            description="P区捺区第五键，包含之字根"
        ),
        KeyRadical(
            key_char="n",
            radicals="已、己、巳、尸、心、羽",
            formula="已半巳满不出己，左框折尸心和羽",
            description="N区折区第一键，包含已字根"
        ),
        KeyRadical(
            key_char="b",
            radicals="子、耳、了、也、卩",
            formula="子耳了也框向上",
            description="B区折区第二键，包含子字根"
        ),
        KeyRadical(
            key_char="v",
            radicals="女、刀、九、臼",
            formula="女刀九臼山朝西",
            description="V区折区第三键，包含女字根"
        ),
        KeyRadical(
            key_char="c",
            radicals="又、巴、马、厶",
            formula="又巴马，丢矢矣",
            description="C区折区第四键，包含又字根"
        ),
        KeyRadical(
            key_char="x",
            radicals="幺、母、弓、匕",
            formula="慈母无心弓和匕，幼无力",
            description="X区折区第五键，包含丝字根"
        ),
    ]

    for kr in key_radicals:
        db.add(kr)
    db.commit()
    print(f"Initialized {len(key_radicals)} key radicals")


def init_english_texts(db: Session):
    existing = db.query(EnglishText).count()
    if existing > 0:
        return

    english_texts = [
        EnglishText(
            title="基础练习",
            content="the quick brown fox jumps over the lazy dog",
            difficulty="easy"
        ),
        EnglishText(
            title="常用句子",
            content="hello world this is a typing practice text for english learning",
            difficulty="easy"
        ),
        EnglishText(
            title="进阶练习",
            content="practice makes perfect keep typing to improve your speed and accuracy",
            difficulty="medium"
        ),
        EnglishText(
            title="高级练习",
            content="the five boxing wizards jump quickly at dawn every single day",
            difficulty="hard"
        ),
    ]

    for et in english_texts:
        db.add(et)
    db.commit()
    print(f"Initialized {len(english_texts)} english texts")


def initialize_data():
    db = SessionLocal()
    try:
        init_key_radicals(db)
        init_english_texts(db)
    finally:
        db.close()
