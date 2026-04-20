package com.wubi.client.ui;

import com.wubi.client.api.WubiApiClient;
import com.wubi.client.model.WubiCharacter;
import com.wubi.client.model.KeyRadical;

import javax.swing.*;
import javax.swing.table.DefaultTableModel;
import java.awt.*;
import java.util.List;

public class WubiLookupPanel extends JPanel {

    private JTextField searchField;
    private JButton searchButton;
    private JTextArea resultArea;

    public WubiLookupPanel() {
        initComponents();
    }

    private void initComponents() {
        setLayout(new BorderLayout(10, 10));
        setBorder(BorderFactory.createEmptyBorder(10, 10, 10, 10));

        JPanel topPanel = new JPanel(new FlowLayout(FlowLayout.LEFT));
        topPanel.add(new JLabel("输入汉字:"));
        searchField = new JTextField(20);
        topPanel.add(searchField);
        searchButton = new JButton("查询");
        topPanel.add(searchButton);
        add(topPanel, BorderLayout.NORTH);

        resultArea = new JTextArea();
        resultArea.setFont(new Font("SansSerif", Font.PLAIN, 18));
        resultArea.setEditable(false);
        resultArea.setLineWrap(true);
        resultArea.setWrapStyleWord(true);
        JScrollPane scrollPane = new JScrollPane(resultArea);
        add(scrollPane, BorderLayout.CENTER);

        searchButton.addActionListener(e -> searchCharacter());
        searchField.addActionListener(e -> searchCharacter());
    }

    private void searchCharacter() {
        String character = searchField.getText().trim();
        if (character.isEmpty()) {
            JOptionPane.showMessageDialog(this, "请输入要查询的汉字", "提示", JOptionPane.WARNING_MESSAGE);
            return;
        }

        searchButton.setEnabled(false);
        resultArea.setText("查询中...");

        SwingWorker<String, Void> worker = new SwingWorker<>() {
            @Override
            protected String doInBackground() {
                try {
                    StringBuilder sb = new StringBuilder();
                    List<WubiCharacter> results = WubiApiClient.searchWubiRoot(character);
                    if (results == null || results.isEmpty()) {
                        WubiCharacter wc = WubiApiClient.getWubiCharacter(character);
                        if (wc != null) {
                            results = List.of(wc);
                        }
                    }
                    
                    if (results.isEmpty()) {
                        sb.append("未找到结果\n");
                    } else {
                        for (WubiCharacter wc : results) {
                            sb.append("字符: ").append(wc.getCharacter()).append("\n");
                            sb.append("五笔: ").append(wc.getWubi86()).append("\n");
                            sb.append("拼音: ").append(wc.getPinyin() != null ? wc.getPinyin() : wc.getSimplePinyin()).append("\n");
                            sb.append("字根: ").append(wc.getRadicals()).append("\n");
                            sb.append("说明: ").append(wc.getDescription()).append("\n");
                            sb.append("---\n");
                        }
                    }
                    return sb.toString();
                } catch (Exception ex) {
                    return "查询失败: " + ex.getMessage();
                }
            }

            @Override
            protected void done() {
                searchButton.setEnabled(true);
                try {
                    resultArea.setText(get());
                } catch (Exception ex) {
                    resultArea.setText("查询失败: " + ex.getMessage());
                }
            }
        };
        worker.execute();
    }
}
