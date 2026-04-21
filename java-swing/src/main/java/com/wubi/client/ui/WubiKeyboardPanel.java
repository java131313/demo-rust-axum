package com.wubi.client.ui;

import com.wubi.client.api.WubiApiClient;
import com.wubi.client.model.KeyRadical;

import javax.swing.*;
import java.awt.*;
import java.util.ArrayList;
import java.util.List;

public class WubiKeyboardPanel extends JPanel {

    private JTextArea formulaArea;
    private List<KeyRadical> keyRadicals;

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

    public WubiKeyboardPanel() {
        initComponents();
        loadKeyRadicals();
    }

    private void loadKeyRadicals() {
        formulaArea.setText("正在从后端加载字根数据...");
        new Thread(() -> {
            try {
                keyRadicals = WubiApiClient.getKeyRadicals();
                SwingUtilities.invokeLater(this::showAllFormulas);
            } catch (Exception e) {
                SwingUtilities.invokeLater(() -> {
                    formulaArea.setText("加载字根数据失败：" + e.getMessage() + "\n\n将使用本地默认数据。");
                    // 加载本地默认数据作为后备
                    loadDefaultKeyRadicals();
                    showAllFormulas();
                });
            }
        }).start();
    }

    private void loadDefaultKeyRadicals() {
        // 本地默认数据作为后备
        keyRadicals = new ArrayList<>();
        keyRadicals.add(createKeyRadical(1, "g", "王、一、五、戋", "王旁青头戋（兼）五一", "G区横区第一键"));
        keyRadicals.add(createKeyRadical(2, "f", "土、士、二、干、十、寸、雨", "土士二干十寸雨", "F区横区第二键"));
        keyRadicals.add(createKeyRadical(3, "d", "大、犬、三、古、石、厂", "大犬三（古）石厂", "D区横区第三键"));
        keyRadicals.add(createKeyRadical(4, "s", "木、丁、西", "木丁西", "S区横区第四键"));
        keyRadicals.add(createKeyRadical(5, "a", "工、戈、艹、七、廿", "工戈草头右框七", "A区横区第五键"));
        keyRadicals.add(createKeyRadical(6, "h", "目、止、卜、虍、上", "目具上止卜虎皮", "H区竖区第一键"));
        keyRadicals.add(createKeyRadical(7, "j", "日、早、虫、刂、竖", "日早两竖与虫依", "J区竖区第二键"));
        keyRadicals.add(createKeyRadical(8, "k", "口、川", "口与川，字根稀", "K区竖区第三键"));
        keyRadicals.add(createKeyRadical(9, "l", "田、甲、四、车、囗", "田甲方框四车里", "L区竖区第四键"));
        keyRadicals.add(createKeyRadical(10, "m", "山、由、贝、几", "山由贝，下框几", "M区竖区第五键"));
        keyRadicals.add(createKeyRadical(11, "t", "禾、竹、丿、彳、攵", "禾竹一撇双人立", "T区撇区第一键"));
        keyRadicals.add(createKeyRadical(12, "r", "白、手、斤、牛", "白手看头三二斤", "R区撇区第二键"));
        keyRadicals.add(createKeyRadical(13, "e", "舟、用、月、豕、衣", "舟用乃月豕（家）衣", "E区撇区第三键"));
        keyRadicals.add(createKeyRadical(14, "w", "人、八、亻", "人八登头单人几", "W区撇区第四键"));
        keyRadicals.add(createKeyRadical(15, "q", "金、饣、勹、儿、夕", "金勺缺点无尾鱼，犬旁留叉", "Q区撇区第五键"));
        keyRadicals.add(createKeyRadical(16, "y", "言、文、方、广、丶", "言文方广在四一，高头一捺谁人去", "Y区捺区第一键"));
        keyRadicals.add(createKeyRadical(17, "u", "立、辛、六、门、疒", "立辛两点六门疒（病）", "U区捺区第二键"));
        keyRadicals.add(createKeyRadical(18, "i", "氵（三点水）、小", "水旁兴头小倒立", "I区捺区第三键"));
        keyRadicals.add(createKeyRadical(19, "o", "火、米、灬", "火业头，四点米", "O区捺区第四键"));
        keyRadicals.add(createKeyRadical(20, "p", "之、宀（宝盖）、冖、礻、衤", "之字军盖建道底，摘礻衤", "P区捺区第五键"));
        keyRadicals.add(createKeyRadical(21, "n", "已、己、巳、尸、心、羽", "已半巳满不出己，左框折尸心和羽", "N区折区第一键"));
        keyRadicals.add(createKeyRadical(22, "b", "子、耳、了、也、卩", "子耳了也框向上", "B区折区第二键"));
        keyRadicals.add(createKeyRadical(23, "v", "女、刀、九、臼", "女刀九臼山朝西", "V区折区第三键"));
        keyRadicals.add(createKeyRadical(24, "c", "又、巴、马、厶", "又巴马，丢矢矣", "C区折区第四键"));
        keyRadicals.add(createKeyRadical(25, "x", "幺、母、弓、匕", "慈母无心弓和匕，幼无力", "X区折区第五键"));
    }

    private KeyRadical createKeyRadical(int id, String keyChar, String radicals, String formula, String description) {
        KeyRadical kr = new KeyRadical();
        kr.setId(id);
        kr.setKeyChar(keyChar);
        kr.setRadicals(radicals);
        kr.setFormula(formula);
        kr.setDescription(description);
        return kr;
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

    private KeyRadical findByKey(String key) {
        if (keyRadicals == null) return null;
        for (KeyRadical kr : keyRadicals) {
            if (kr.getKeyChar().equalsIgnoreCase(key)) {
                return kr;
            }
        }
        return null;
    }

    private void showKeyFormula(String key) {
        if (keyRadicals == null) {
            formulaArea.setText("字根数据尚未加载，请稍后再试");
            return;
        }

        KeyRadical kr = findByKey(key);
        if (kr == null) {
            formulaArea.setText("未找到键位 " + key + " 的字根信息");
            return;
        }

        StringBuilder sb = new StringBuilder();
        sb.append("=== ").append(key.toUpperCase()).append(" 键 ===\n\n");
        sb.append("【口诀】").append(kr.getFormula()).append("\n\n");
        sb.append("【主要字根】").append(kr.getRadicals()).append("\n\n");

        String keyChar = kr.getFormula().substring(0, 1);
        sb.append("【键名字】").append(keyChar).append("\n\n");
        sb.append("【学习提示】键名字就是口诀第一个字\n");
        sb.append("【说明】").append(kr.getDescription()).append("\n");

        formulaArea.setText(sb.toString());
    }

    private void showAllFormulas() {
        if (keyRadicals == null) {
            formulaArea.setText("字根数据尚未加载，请稍后再试");
            return;
        }

        StringBuilder sb = new StringBuilder();
        String[] zoneNames = {"第一区：横起笔 (G - A)", "第二区：竖起笔 (H - M)",
                              "第三区：撇起笔 (T - Q)", "第四区：捺/点起笔 (Y - P)",
                              "第五区：折起笔 (N - X)"};
        String[] zoneKeys = {"gfdsa", "hjklm", "trweq", "yuiop", "nbvcx"};

        for (int z = 0; z < 5; z++) {
            sb.append("【").append(zoneNames[z]).append("】\n");
            sb.append("-".repeat(50)).append("\n");

            for (char c : zoneKeys[z].toCharArray()) {
                KeyRadical kr = findByKey(String.valueOf(c));
                if (kr != null) {
                    sb.append(String.format("  %s 键 | %s | 字根: %s\n",
                        kr.getKeyChar().toUpperCase(), kr.getFormula(), kr.getRadicals()));
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
}
