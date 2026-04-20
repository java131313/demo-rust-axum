package com.wubi.client.ui;

import com.wubi.client.api.WubiApiClient;

import javax.swing.*;
import java.awt.*;
import java.awt.event.KeyAdapter;
import java.awt.event.KeyEvent;
import java.util.ArrayList;
import java.util.List;

public class EnglishTypingPanel extends JPanel {

    private JTextArea textArea;
    private JTextField inputField;
    private JLabel wpmLabel;
    private JLabel accuracyLabel;
    private JLabel timeLabel;

    private String currentText = "";
    private int currentIndex = 0;
    private long startTime = 0;
    private int totalChars = 0;
    private int correctChars = 0;

    public EnglishTypingPanel() {
        initComponents();
        loadNewText();
    }

    private void initComponents() {
        setLayout(new BorderLayout(10, 10));
        setBorder(BorderFactory.createEmptyBorder(10, 10, 10, 10));

        JPanel statsPanel = new JPanel(new FlowLayout(FlowLayout.LEFT));
        wpmLabel = new JLabel("WPM: 0");
        statsPanel.add(wpmLabel);
        accuracyLabel = new JLabel("正确率: 0%");
        statsPanel.add(accuracyLabel);
        timeLabel = new JLabel("时间: 0s");
        statsPanel.add(timeLabel);

        JButton refreshButton = new JButton("刷新文章");
        statsPanel.add(refreshButton);
        add(statsPanel, BorderLayout.NORTH);

        textArea = new JTextArea(10, 50);
        textArea.setFont(new Font("Monospaced", Font.PLAIN, 16));
        textArea.setEditable(false);
        textArea.setLineWrap(true);
        textArea.setWrapStyleWord(true);
        JScrollPane scrollPane = new JScrollPane(textArea);
        add(scrollPane, BorderLayout.CENTER);

        JPanel inputPanel = new JPanel(new BorderLayout());
        inputPanel.add(new JLabel("输入:"), BorderLayout.WEST);
        inputField = new JTextField(30);
        inputField.setFont(new Font("Monospaced", Font.PLAIN, 16));
        inputField.addKeyListener(new KeyAdapter() {
            @Override
            public void keyTyped(KeyEvent e) {
                char c = e.getKeyChar();
                if (startTime == 0) {
                    startTime = System.currentTimeMillis();
                    startTimer();
                }
                checkInput(c);
            }
        });
        inputPanel.add(inputField, BorderLayout.CENTER);
        add(inputPanel, BorderLayout.SOUTH);

        refreshButton.addActionListener(e -> loadNewText());
    }

    private void loadNewText() {
        SwingWorker<String, Void> worker = new SwingWorker<>() {
            @Override
            protected String doInBackground() {
                try {
                    String response = WubiApiClient.getEnglishTexts();
                    List<String> texts = parseJsonArray(response);
                    if (!texts.isEmpty()) {
                        return texts.get((int)(Math.random() * texts.size()));
                    }
                } catch (Exception ex) {
                }
                return "The quick brown fox jumps over the lazy dog. Programming is fun and rewarding. Practice makes perfect.";
            }

            @Override
            protected void done() {
                try {
                    currentText = get();
                    textArea.setText(currentText);
                    currentIndex = 0;
                    totalChars = 0;
                    correctChars = 0;
                    startTime = 0;
                    inputField.setText("");
                    updateStats();
                } catch (Exception ex) {
                    textArea.setText("加载失败，使用默认文本");
                }
            }
        };
        worker.execute();
    }

    private List<String> parseJsonArray(String json) {
        List<String> result = new ArrayList<>();
        int idx = 0;
        while ((idx = json.indexOf('{', idx)) != -1) {
            int end = findMatchingBrace(json, idx);
            if (end == -1) break;
            String item = json.substring(idx, end + 1);
            String content = extractString(item, "content");
            if (content != null) result.add(content);
            idx = end + 1;
        }
        return result;
    }

    private String extractString(String json, String key) {
        String search = "\"" + key + "\":";
        int idx = json.indexOf(search);
        if (idx == -1) {
            search = "\"" + key + "\" :";
            idx = json.indexOf(search);
        }
        if (idx == -1) return null;
        idx += search.length();
        while (idx < json.length() && json.charAt(idx) <= ' ') idx++;
        if (idx >= json.length() || json.charAt(idx) != '"') return null;
        int start = idx + 1;
        int end = start;
        while (end < json.length()) {
            if (json.charAt(end) == '\\' && end + 1 < json.length()) {
                end += 2;
                continue;
            }
            if (json.charAt(end) == '"') break;
            end++;
        }
        return json.substring(start, end);
    }

    private int findMatchingBrace(String json, int start) {
        int depth = 0;
        for (int i = start; i < json.length(); i++) {
            char c = json.charAt(i);
            if (c == '"') {
                i++;
                while (i < json.length()) {
                    if (json.charAt(i) == '\\' && i + 1 < json.length()) {
                        i += 2;
                        continue;
                    }
                    if (json.charAt(i) == '"') break;
                    i++;
                }
            } else if (c == '{') {
                depth++;
            } else if (c == '}') {
                depth--;
                if (depth == 0) return i;
            }
        }
        return -1;
    }

    private void checkInput(char c) {
        if (currentIndex >= currentText.length()) return;

        totalChars++;
        if (c == currentText.charAt(currentIndex)) {
            correctChars++;
        }
        currentIndex++;

        updateStats();

        if (currentIndex >= currentText.length()) {
            JOptionPane.showMessageDialog(this, "完成!");
            loadNewText();
        }
    }

    private void updateStats() {
        long elapsed = startTime > 0 ? System.currentTimeMillis() - startTime : 0;
        double minutes = elapsed / 60000.0;
        int wpm = minutes > 0 ? (int) (correctChars / 5.0 / minutes) : 0;
        double accuracy = totalChars > 0 ? (double) correctChars / totalChars * 100 : 0;

        wpmLabel.setText("WPM: " + wpm);
        accuracyLabel.setText(String.format("正确率: %.1f%%", accuracy));
        timeLabel.setText("时间: " + (elapsed / 1000) + "s");
    }

    private void startTimer() {
        Timer timer = new Timer(1000, e -> {
            long elapsed = System.currentTimeMillis() - startTime;
            timeLabel.setText("时间: " + (elapsed / 1000) + "s");
        });
        timer.start();
    }
}
