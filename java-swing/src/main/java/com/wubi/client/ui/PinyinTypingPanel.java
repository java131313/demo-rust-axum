package com.wubi.client.ui;

import com.wubi.client.api.WubiApiClient;
import com.wubi.client.model.WubiCharacter;

import javax.swing.*;
import java.awt.*;
import java.awt.event.KeyAdapter;
import java.awt.event.KeyEvent;

public class PinyinTypingPanel extends JPanel {

    private JLabel charLabel;
    private JLabel pinyinLabel;
    private JTextField inputField;
    private JLabel feedbackLabel;

    private String currentChar = "你";
    private String currentPinyin = "ni";
    private int totalChars = 0;
    private int correctChars = 0;

    public PinyinTypingPanel() {
        initComponents();
        loadNextChar();
    }

    private void initComponents() {
        setLayout(new BorderLayout(10, 10));
        setBorder(BorderFactory.createEmptyBorder(10, 10, 10, 10));

        JPanel centerPanel = new JPanel(new GridBagLayout());
        GridBagConstraints gbc = new GridBagConstraints();
        gbc.insets = new Insets(10, 10, 10, 10);

        // Character display
        charLabel = new JLabel("", SwingConstants.CENTER);
        charLabel.setFont(new Font("SansSerif", Font.BOLD, 72));
        gbc.gridx = 0; gbc.gridy = 0;
        centerPanel.add(charLabel, gbc);

        // Pinyin display
        pinyinLabel = new JLabel("", SwingConstants.CENTER);
        pinyinLabel.setFont(new Font("SansSerif", Font.BOLD, 36));
        pinyinLabel.setForeground(Color.BLUE);
        gbc.gridx = 0; gbc.gridy = 1;
        centerPanel.add(pinyinLabel, gbc);

        // Input field
        inputField = new JTextField(20);
        inputField.setFont(new Font("SansSerif", Font.PLAIN, 24));
        inputField.setHorizontalAlignment(JTextField.CENTER);
        inputField.addKeyListener(new KeyAdapter() {
            @Override
            public void keyPressed(KeyEvent e) {
                if (e.getKeyCode() == KeyEvent.VK_ENTER) {
                    checkInput();
                }
            }
        });
        gbc.gridx = 0; gbc.gridy = 2;
        centerPanel.add(inputField, gbc);

        // Feedback
        feedbackLabel = new JLabel("输入拼音后按回车", SwingConstants.CENTER);
        feedbackLabel.setFont(new Font("SansSerif", Font.PLAIN, 18));
        gbc.gridx = 0; gbc.gridy = 3;
        centerPanel.add(feedbackLabel, gbc);

        add(centerPanel, BorderLayout.CENTER);

        // Bottom stats
        JPanel bottomPanel = new JPanel(new FlowLayout());
        JButton nextButton = new JButton("下一个");
        JButton hintButton = new JButton("提示");
        JLabel statsLabel = new JLabel("正确率: 0%");

        nextButton.addActionListener(e -> loadNextChar());
        hintButton.addActionListener(e -> pinyinLabel.setVisible(!pinyinLabel.isVisible()));
        add(bottomPanel, BorderLayout.SOUTH);
    }

    private void loadNextChar() {
        String[] chars = {"你", "好", "我", "他", "她", "人", "口", "日", "月", "水", "火", "土"};
        currentChar = chars[(int) (Math.random() * chars.length)];
        charLabel.setText(currentChar);
        pinyinLabel.setText("???");
        pinyinLabel.setVisible(false);
        inputField.setText("");
        feedbackLabel.setText("输入拼音后按回车");
        feedbackLabel.setForeground(Color.BLACK);

        SwingWorker<String, Void> worker = new SwingWorker<>() {
            @Override
            protected String doInBackground() {
                try {
                    WubiCharacter wc = WubiApiClient.getWubiCharacter(currentChar);
                    return wc.getPinyin() != null ? wc.getPinyin() : wc.getSimplePinyin();
                } catch (Exception e) {
                    return getFallbackPinyin(currentChar);
                }
            }

            @Override
            protected void done() {
                try {
                    currentPinyin = get();
                    pinyinLabel.setText(currentPinyin);
                } catch (Exception ex) {
                    currentPinyin = getFallbackPinyin(currentChar);
                    pinyinLabel.setText(currentPinyin);
                }
            }
        };
        worker.execute();
    }

    private void checkInput() {
        String input = inputField.getText().trim().toLowerCase();
        if (input.isEmpty()) return;

        totalChars++;
        if (input.equals(currentPinyin) || input.equals(currentPinyin.replaceAll("[āáǎàēéěèīíǐìōóǒòūúǔùǖǘǚǜ]", "aeiou"))) {
            correctChars++;
            feedbackLabel.setText("正确!");
            feedbackLabel.setForeground(new Color(0, 150, 0));
        } else {
            feedbackLabel.setText("错误，正确拼音: " + currentPinyin);
            feedbackLabel.setForeground(Color.RED);
        }
        inputField.setText("");
    }

    private String getFallbackPinyin(String c) {
        switch (c) {
            case "你": return "ni";
            case "好": return "hao";
            case "我": return "wo";
            case "他": return "ta";
            case "她": return "ta";
            case "人": return "ren";
            case "口": return "kou";
            case "日": return "ri";
            case "月": return "yue";
            case "水": return "shui";
            case "火": return "huo";
            case "土": return "tu";
            default: return "未知";
        }
    }
}
