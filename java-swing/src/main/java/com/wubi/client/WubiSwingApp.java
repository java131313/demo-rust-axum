package com.wubi.client;

import com.wubi.client.api.WubiApiClient;
import com.wubi.client.ui.LoginDialog;
import com.wubi.client.ui.MainWindow;

import javax.swing.*;
import java.awt.*;

public class WubiSwingApp {

    public static void main(String[] args) {
        SwingUtilities.invokeLater(() -> {
            try {
                UIManager.setLookAndFeel(UIManager.getSystemLookAndFeelClassName());
            } catch (Exception e) {
                e.printStackTrace();
            }

            JFrame frame = new JFrame("五笔打字练习系统 - 加载中");
            frame.setSize(400, 150);
            frame.setLocationRelativeTo(null);
            frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);

            JLabel label = new JLabel("正在连接后端服务器...", SwingConstants.CENTER);
            label.setFont(new Font("SansSerif", Font.PLAIN, 16));
            frame.add(label);
            frame.setVisible(true);

            SwingWorker<Boolean, Void> worker = new SwingWorker<>() {
                @Override
                protected Boolean doInBackground() {
                    return WubiApiClient.isHealthy();
                }

                @Override
                protected void done() {
                    frame.dispose();
                    try {
                        boolean healthy = get();
                        if (healthy) {
                            MainWindow main = new MainWindow();
                            main.setVisible(true);
                        } else {
                            JOptionPane.showMessageDialog(null,
                                    "无法连接后端服务器 (http://localhost:3000)\n\n" +
                                            "请确保 Rust 后端已启动:\n" +
                                            "cargo run --release",
                                    "连接失败",
                                    JOptionPane.ERROR_MESSAGE);
                            System.exit(1);
                        }
                    } catch (Exception e) {
                        JOptionPane.showMessageDialog(null,
                                "启动失败: " + e.getMessage(),
                                "错误",
                                JOptionPane.ERROR_MESSAGE);
                        System.exit(1);
                    }
                }
            };
            worker.execute();
        });
    }
}
