package com.wubi.config;

import com.wubi.entity.KeyRadical;
import com.wubi.entity.EnglishText;
import com.wubi.repository.KeyRadicalRepository;
import com.wubi.repository.EnglishTextRepository;
import org.springframework.boot.CommandLineRunner;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class DataInitializer {

    @Bean
    public CommandLineRunner initData(
            KeyRadicalRepository keyRadicalRepository,
            EnglishTextRepository englishTextRepository) {
        return args -> {
            if (keyRadicalRepository.count() == 0) {
                initKeyRadicals(keyRadicalRepository);
            }

            if (englishTextRepository.count() == 0) {
                initEnglishTexts(englishTextRepository);
            }
        };
    }

    private void initKeyRadicals(KeyRadicalRepository repository) {
        KeyRadical[] keyRadicals = new KeyRadical[25];
        
        // G区
        keyRadicals[0] = new KeyRadical();
        keyRadicals[0].setKeyChar("g");
        keyRadicals[0].setRadicals("王、一、五、戋");
        keyRadicals[0].setFormula("王旁青头戋（兼）五一");
        keyRadicals[0].setDescription("G区横区第一键，包含横笔和戈字根");
        
        keyRadicals[1] = new KeyRadical();
        keyRadicals[1].setKeyChar("f");
        keyRadicals[1].setRadicals("土、士、二、干，十、寸、雨");
        keyRadicals[1].setFormula("土士二干十寸雨");
        keyRadicals[1].setDescription("F区横区第二键，包含土字根");
        
        keyRadicals[2] = new KeyRadical();
        keyRadicals[2].setKeyChar("d");
        keyRadicals[2].setRadicals("大、犬、三、古、石、厂");
        keyRadicals[2].setFormula("大犬三（古）石厂");
        keyRadicals[2].setDescription("D区横区第三键，包含大字根");
        
        keyRadicals[3] = new KeyRadical();
        keyRadicals[3].setKeyChar("s");
        keyRadicals[3].setRadicals("木、丁、西");
        keyRadicals[3].setFormula("木丁西");
        keyRadicals[3].setDescription("S区横区第四键，包含木字根");
        
        keyRadicals[4] = new KeyRadical();
        keyRadicals[4].setKeyChar("a");
        keyRadicals[4].setRadicals("工、戈、艹、七、廿");
        keyRadicals[4].setFormula("工戈草头右框七");
        keyRadicals[4].setDescription("A区横区第五键，包含工字根");
        
        // H区
        keyRadicals[5] = new KeyRadical();
        keyRadicals[5].setKeyChar("h");
        keyRadicals[5].setRadicals("目、止、卜、虍、上");
        keyRadicals[5].setFormula("目具上止卜虎皮");
        keyRadicals[5].setDescription("H区竖区第一键，包含目字根");
        
        keyRadicals[6] = new KeyRadical();
        keyRadicals[6].setKeyChar("j");
        keyRadicals[6].setRadicals("日、早、虫、刂、竖");
        keyRadicals[6].setFormula("日早两竖与虫依");
        keyRadicals[6].setDescription("J区竖区第二键，包含日字根");
        
        keyRadicals[7] = new KeyRadical();
        keyRadicals[7].setKeyChar("k");
        keyRadicals[7].setRadicals("口、川");
        keyRadicals[7].setFormula("口与川，字根稀");
        keyRadicals[7].setDescription("K区竖区第三键，包含口字根");
        
        keyRadicals[8] = new KeyRadical();
        keyRadicals[8].setKeyChar("l");
        keyRadicals[8].setRadicals("田、甲、四、车、囗");
        keyRadicals[8].setFormula("田甲方框四车里");
        keyRadicals[8].setDescription("L区竖区第四键，包含田字根");
        
        keyRadicals[9] = new KeyRadical();
        keyRadicals[9].setKeyChar("m");
        keyRadicals[9].setRadicals("山、由、贝、几");
        keyRadicals[9].setFormula("山由贝，下框几");
        keyRadicals[9].setDescription("M区竖区第五键，包含山字根");
        
        // T区
        keyRadicals[10] = new KeyRadical();
        keyRadicals[10].setKeyChar("t");
        keyRadicals[10].setRadicals("禾、竹、丿、彳、攵");
        keyRadicals[10].setFormula("禾竹一撇双人立");
        keyRadicals[10].setDescription("T区撇区第一键，包含禾字根");
        
        keyRadicals[11] = new KeyRadical();
        keyRadicals[11].setKeyChar("r");
        keyRadicals[11].setRadicals("白、手、斤、牛");
        keyRadicals[11].setFormula("白手看头三二斤");
        keyRadicals[11].setDescription("R区撇区第二键，包含白字根");
        
        keyRadicals[12] = new KeyRadical();
        keyRadicals[12].setKeyChar("e");
        keyRadicals[12].setRadicals("舟、用、月、豕、衣");
        keyRadicals[12].setFormula("舟用乃月豕（家）衣");
        keyRadicals[12].setDescription("E区撇区第三键，包含月字根");
        
        keyRadicals[13] = new KeyRadical();
        keyRadicals[13].setKeyChar("w");
        keyRadicals[13].setRadicals("人、八、亻");
        keyRadicals[13].setFormula("人八登头单人几");
        keyRadicals[13].setDescription("W区撇区第四键，包含人字根");
        
        keyRadicals[14] = new KeyRadical();
        keyRadicals[14].setKeyChar("q");
        keyRadicals[14].setRadicals("金、饣、勹、儿、夕");
        keyRadicals[14].setFormula("金勺缺点无尾鱼，犬旁留叉");
        keyRadicals[14].setDescription("Q区撇区第五键，包含金字根");
        
        // Y区
        keyRadicals[15] = new KeyRadical();
        keyRadicals[15].setKeyChar("y");
        keyRadicals[15].setRadicals("言、文、方、广、丶");
        keyRadicals[15].setFormula("言文方广在四一，高头一捺谁人去");
        keyRadicals[15].setDescription("Y区捺区第一键，包含言字根");
        
        keyRadicals[16] = new KeyRadical();
        keyRadicals[16].setKeyChar("u");
        keyRadicals[16].setRadicals("立、辛、六、门、疒");
        keyRadicals[16].setFormula("立辛两点六门疒（病）");
        keyRadicals[16].setDescription("U区捺区第二键，包含立字根");
        
        keyRadicals[17] = new KeyRadical();
        keyRadicals[17].setKeyChar("i");
        keyRadicals[17].setRadicals("氵（三点水）、小");
        keyRadicals[17].setFormula("水旁兴头小倒立");
        keyRadicals[17].setDescription("I区捺区第三键，包含水字根");
        
        keyRadicals[18] = new KeyRadical();
        keyRadicals[18].setKeyChar("o");
        keyRadicals[18].setRadicals("火、米、灬");
        keyRadicals[18].setFormula("火业头，四点米");
        keyRadicals[18].setDescription("O区捺区第四键，包含火字根");
        
        keyRadicals[19] = new KeyRadical();
        keyRadicals[19].setKeyChar("p");
        keyRadicals[19].setRadicals("之、宀（宝盖）、冖、礻、衤");
        keyRadicals[19].setFormula("之字军盖建道底，摘礻衤");
        keyRadicals[19].setDescription("P区捺区第五键，包含之字根");
        
        // N区
        keyRadicals[20] = new KeyRadical();
        keyRadicals[20].setKeyChar("n");
        keyRadicals[20].setRadicals("已、己、巳、尸、心、羽");
        keyRadicals[20].setFormula("已半巳满不出己，左框折尸心和羽");
        keyRadicals[20].setDescription("N区折区第一键，包含已字根");
        
        keyRadicals[21] = new KeyRadical();
        keyRadicals[21].setKeyChar("b");
        keyRadicals[21].setRadicals("子、耳、了、也、卩");
        keyRadicals[21].setFormula("子耳了也框向上");
        keyRadicals[21].setDescription("B区折区第二键，包含子字根");
        
        keyRadicals[22] = new KeyRadical();
        keyRadicals[22].setKeyChar("v");
        keyRadicals[22].setRadicals("女、刀、九、臼");
        keyRadicals[22].setFormula("女刀九臼山朝西");
        keyRadicals[22].setDescription("V区折区第三键，包含女字根");
        
        keyRadicals[23] = new KeyRadical();
        keyRadicals[23].setKeyChar("c");
        keyRadicals[23].setRadicals("又、巴、马、厶");
        keyRadicals[23].setFormula("又巴马，丢矢矣");
        keyRadicals[23].setDescription("C区折区第四键，包含又字根");
        
        keyRadicals[24] = new KeyRadical();
        keyRadicals[24].setKeyChar("x");
        keyRadicals[24].setRadicals("幺、母、弓、匕");
        keyRadicals[24].setFormula("慈母无心弓和匕，幼无力");
        keyRadicals[24].setDescription("X区折区第五键，包含丝字根");

        for (KeyRadical kr : keyRadicals) {
            repository.save(kr);
        }
    }

    private void initEnglishTexts(EnglishTextRepository repository) {
        EnglishText[] englishTexts = new EnglishText[4];
        
        englishTexts[0] = new EnglishText();
        englishTexts[0].setTitle("基础练习");
        englishTexts[0].setContent("the quick brown fox jumps over the lazy dog");
        englishTexts[0].setDifficulty("easy");
        
        englishTexts[1] = new EnglishText();
        englishTexts[1].setTitle("常用句子");
        englishTexts[1].setContent("hello world this is a typing practice text for english learning");
        englishTexts[1].setDifficulty("easy");
        
        englishTexts[2] = new EnglishText();
        englishTexts[2].setTitle("进阶练习");
        englishTexts[2].setContent("practice makes perfect keep typing to improve your speed and accuracy");
        englishTexts[2].setDifficulty("medium");
        
        englishTexts[3] = new EnglishText();
        englishTexts[3].setTitle("高级练习");
        englishTexts[3].setContent("the five boxing wizards jump quickly at dawn every single day");
        englishTexts[3].setDifficulty("hard");

        for (EnglishText et : englishTexts) {
            repository.save(et);
        }
    }
}
