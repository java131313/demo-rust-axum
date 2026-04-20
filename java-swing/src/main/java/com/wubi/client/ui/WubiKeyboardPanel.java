package com.wubi.client.ui;

import javax.swing.*;
import java.awt.*;

public class WubiKeyboardPanel extends JPanel {

    private JTextArea formulaArea;

    private final String[][] KEYBOARD_LAYOUT = {
        {"G", "H", "T", "Y", "N"},
        {"F", "J", "R", "U", "B"},
        {"D", "K", "E", "I", "V"},
        {"S", "L", "W", "O", "C"},
        {"A", "M", "Q", "P", "X"}
    };

    private final String[] ZONE_NAMES = {"横区(1)", "竖区(2)", "撇区(3)", "捺区(4)", "折区(5)"};
    private final Color[] ZONE_COLORS = {
        new Color(239, 68, 68),
        new Color(245, 158, 11),
        new Color(34, 197, 94),
        new Color(59, 130, 246),
        new Color(168, 85, 247)
    };

    private final KeyRadicalData[] KEY_RADICALS = {
        new KeyRadicalData("g", "11", "王旁青头戋（兼）五一", "王、一、五、戋"),
        new KeyRadicalData("f", "12", "土士二干十寸雨", "土、士、二、干、十、寸、雨"),
        new KeyRadicalData("d", "13", "大犬三（古）石厂", "大、犬、三、古、石、厂"),
        new KeyRadicalData("s", "14", "木丁西", "木、丁、西"),
        new KeyRadicalData("a", "15", "工戈草头右框七", "工、戈、艹、七、廿"),
        new KeyRadicalData("h", "21", "目具上止卜虎皮", "目、止、卜、虍、上"),
        new KeyRadicalData("j", "22", "日早两竖与虫依", "日、早、虫、刂、竖"),
        new KeyRadicalData("k", "23", "口与川，字根稀", "口、川"),
        new KeyRadicalData("l", "24", "田甲方框四车里", "田、甲、四、车、囗"),
        new KeyRadicalData("m", "25", "山由贝，下框几", "山、由、贝、几"),
        new KeyRadicalData("t", "31", "禾竹一撇双人立", "禾、竹、丿、彳、攵"),
        new KeyRadicalData("r", "32", "白手看头三二斤", "白、手、斤、牛"),
        new KeyRadicalData("e", "33", "舟用乃月豕（家）衣", "舟、用、月、豕、衣"),
        new KeyRadicalData("w", "34", "人八登头单人几", "人、八、亻"),
        new KeyRadicalData("q", "35", "金勺缺点无尾鱼，犬旁留叉", "金、饣、勹、儿、夕"),
        new KeyRadicalData("y", "41", "言文方广在四一，高头一捺谁人去", "言、文、方、广、丶"),
        new KeyRadicalData("u", "42", "立辛两点六门疒（病）", "立、辛、六、门、疒"),
        new KeyRadicalData("i", "43", "水旁兴头小倒立", "氵（三点水）、小"),
        new KeyRadicalData("o", "44", "火业头，四点米", "火、米、灬"),
        new KeyRadicalData("p", "45", "之字军盖建道底，摘礻衤", "之、宀（宝盖）、冖、礻、衤"),
        new KeyRadicalData("n", "51", "已半巳满不出己，左框折尸心和羽", "已、己、巳、尸、心、羽"),
        new KeyRadicalData("b", "52", "子耳了也框向上", "子、耳、了、也、卩"),
        new KeyRadicalData("v", "53", "女刀九臼山朝西", "女、刀、九、臼"),
        new KeyRadicalData("c", "54", "又巴马，丢矢矣", "又、巴、马、厶"),
        new KeyRadicalData("x", "55", "慈母无心弓和匕，幼无力", "幺、母、弓、匕"),
    };

    public WubiKeyboardPanel() {
        initComponents();
        showAllFormulas();
    }

    private void initComponents() {
        setLayout(new BorderLayout());
        setBorder(BorderFactory.createEmptyBorder(10, 10, 10, 10));

        JPanel keyboardPanel = new JPanel(new BorderLayout());
        keyboardPanel.setBorder(BorderFactory.createTitledBorder("五笔键盘"));

        JPanel gridPanel = new JPanel(new GridLayout(0, 5, 5, 5));
        gridPanel.setBorder(BorderFactory.createEmptyBorder(10, 10, 10, 10));

        for (int i = 0; i < 5; i++) {
            JLabel label = new JLabel(ZONE_NAMES[i], SwingConstants.CENTER);
            label.setOpaque(true);
            label.setBackground(ZONE_COLORS[i]);
            label.setForeground(Color.WHITE);
            label.setFont(new Font("SansSerif", Font.BOLD, 14));
            gridPanel.add(label);
        }

        for (String[] row : KEYBOARD_LAYOUT) {
            for (String key : row) {
                JButton keyBtn = new JButton(key);
                keyBtn.setFont(new Font("SansSerif", Font.BOLD, 18));
                keyBtn.setPreferredSize(new Dimension(80, 60));
                keyBtn.addActionListener(e -> showKeyFormula(key));
                gridPanel.add(keyBtn);
            }
        }

        keyboardPanel.add(gridPanel, BorderLayout.CENTER);
        add(keyboardPanel, BorderLayout.NORTH);

        formulaArea = new JTextArea(15, 50);
        formulaArea.setFont(new Font("SansSerif", Font.PLAIN, 15));
        formulaArea.setEditable(false);
        formulaArea.setLineWrap(true);
        formulaArea.setWrapStyleWord(true);
        JScrollPane scrollPane = new JScrollPane(formulaArea);
        scrollPane.setBorder(BorderFactory.createTitledBorder("字根口诀"));
        add(scrollPane, BorderLayout.CENTER);
    }

    private KeyRadicalData findByKey(String key) {
        for (KeyRadicalData kr : KEY_RADICALS) {
            if (kr.key.equalsIgnoreCase(key)) {
                return kr;
            }
        }
        return null;
    }

    private void showKeyFormula(String key) {
        KeyRadicalData kr = findByKey(key);
        if (kr == null) {
            formulaArea.setText("未找到键位 " + key + " 的字根信息");
            return;
        }

        StringBuilder sb = new StringBuilder();
        sb.append("=== ").append(key.toUpperCase()).append(" 键 (编码 ").append(kr.code).append(") ===\n\n");
        sb.append("【口诀】").append(kr.formula).append("\n\n");
        sb.append("【主要字根】").append(kr.radicals).append("\n\n");

        String keyChar = kr.formula.substring(0, 1);
        sb.append("【键名字】").append(keyChar).append("\n\n");
        sb.append("【学习提示】键名字就是口诀第一个字\n");

        formulaArea.setText(sb.toString());
    }

    private void showAllFormulas() {
        StringBuilder sb = new StringBuilder();
        String[] zoneNames = {"第一区：横起笔 (G - A)", "第二区：竖起笔 (H - M)",
                              "第三区：撇起笔 (T - Q)", "第四区：捺/点起笔 (Y - P)",
                              "第五区：折起笔 (N - X)"};
        String[] zoneKeys = {"gfdsa", "hjklm", "trweq", "yuiop", "nbvcx"};

        for (int z = 0; z < 5; z++) {
            sb.append("【").append(zoneNames[z]).append("】\n");
            sb.append("-".repeat(50)).append("\n");

            for (char c : zoneKeys[z].toCharArray()) {
                KeyRadicalData kr = findByKey(String.valueOf(c));
                if (kr != null) {
                    sb.append(String.format("  %s 键 | %s | 字根: %s\n",
                        kr.key.toUpperCase(), kr.formula, kr.radicals));
                }
            }
            sb.append("\n");
        }

        sb.append("【学习小贴士】\n");
        sb.append("-".repeat(50)).append("\n");
        sb.append("• 键名字：每个键位上的口诀第一个字就是[键名字]（如 G 是王、F 是土）\n");
        sb.append("• 成字字根：字根本身也是一个汉字的（如[石]、[手]、[口]）\n");
        sb.append("• 拆分原则：书写顺序、取大优先、兼顾直观、能连不交\n");

        formulaArea.setText(sb.toString());
    }

    private static class KeyRadicalData {
        String key;
        String code;
        String formula;
        String radicals;

        KeyRadicalData(String key, String code, String formula, String radicals) {
            this.key = key;
            this.code = code;
            this.formula = formula;
            this.radicals = radicals;
        }
    }
}
