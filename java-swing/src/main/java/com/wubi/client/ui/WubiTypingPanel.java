package com.wubi.client.ui;

import javax.swing.*;
import java.awt.*;

public class WubiTypingPanel extends JPanel {

    private JTextArea displayArea;
    private JTextField inputField;
    private JLabel statusLabel;
    private JLabel accuracyLabel;

    private String currentWord = "你好世界";
    private int totalChars = 0;
    private int correctChars = 0;

    public WubiTypingPanel() {
        initComponents();
    }

    private void initComponents() {
        setLayout(new BorderLayout(10, 10));
        setBorder(BorderFactory.createEmptyBorder(10, 10, 10, 10));

        // Top info panel
        JPanel topPanel = new JPanel(new FlowLayout(FlowLayout.LEFT));
        topPanel.add(new JLabel("当前汉字:"));
        JLabel charLabel = new JLabel(currentWord);
        charLabel.setFont(new Font("SansSerif", Font.BOLD, 32));
        topPanel.add(charLabel);

        JButton nextButton = new JButton("下一个");
        topPanel.add(nextButton);
        add(topPanel, BorderLayout.NORTH);

        // Input area
        JPanel inputPanel = new JPanel(new BorderLayout(5, 5));
        inputPanel.add(new JLabel("输入五笔编码:"), BorderLayout.NORTH);
        inputField = new JTextField(30);
        inputField.setFont(new Font("Monospaced", Font.PLAIN, 20));
        inputPanel.add(inputField, BorderLayout.CENTER);
        add(inputPanel, BorderLayout.CENTER);

        // Status area
        JPanel statusPanel = new JPanel(new FlowLayout(FlowLayout.LEFT));
        statusLabel = new JLabel("状态: 准备就绪");
        statusPanel.add(statusLabel);
        accuracyLabel = new JLabel("正确率: 0%");
        statusPanel.add(accuracyLabel);
        add(statusPanel, BorderLayout.SOUTH);

        // Action listeners
        nextButton.addActionListener(e -> nextWord());
        inputField.addActionListener(e -> checkInput());
    }

    private void checkInput() {
        String input = inputField.getText().trim();
        if (input.isEmpty()) return;

        totalChars += currentWord.length();
        if (input.equalsIgnoreCase(currentWord) || input.contains(currentWord)) {
            correctChars += currentWord.length();
            statusLabel.setText("正确!");
            statusLabel.setForeground(new Color(0, 150, 0));
        } else {
            statusLabel.setText("错误，正确五笔: (请查看字根键盘)");
            statusLabel.setForeground(Color.RED);
        }

        double accuracy = totalChars > 0 ? (double) correctChars / totalChars * 100 : 0;
        accuracyLabel.setText(String.format("正确率: %.1f%%", accuracy));

        inputField.setText("");
    }

    private void nextWord() {
        String[] words = {"你好", "世界", "学习", "五笔", "打字", "练习", "中国", "北京", "上海", "广州"};
        currentWord = words[(int) (Math.random() * words.length)];
        statusLabel.setText("新汉字: " + currentWord);
        statusLabel.setForeground(Color.BLACK);
        inputField.setText("");
        inputField.requestFocus();
    }
}
