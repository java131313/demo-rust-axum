package com.wubi.client.ui;

import com.wubi.client.api.WubiApiClient;

import javax.swing.*;
import java.awt.*;
import java.awt.event.WindowAdapter;
import java.awt.event.WindowEvent;

public class MainWindow extends JFrame {

    private JTabbedPane tabbedPane;
    private JLabel statusLabel;
    private JLabel userLabel;

    public MainWindow() {
        super("五笔打字练习系统");
        setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        setSize(1000, 700);
        setLocationRelativeTo(null);

        initComponents();
        setupMenuBar();
    }

    private void initComponents() {
        tabbedPane = new JTabbedPane();
        tabbedPane.addTab("五笔打字练习", new WubiTypingPanel());
        tabbedPane.addTab("汉字五笔查询", new WubiLookupPanel());
        tabbedPane.addTab("字根键盘与口诀", new WubiKeyboardPanel());
        tabbedPane.addTab("英语打字练习", new EnglishTypingPanel());
        tabbedPane.addTab("拼音打字练习", new PinyinTypingPanel());

        add(tabbedPane, BorderLayout.CENTER);

        statusLabel = new JLabel("状态: 已连接");
        userLabel = new JLabel("未登录");
        JPanel statusBar = new JPanel(new BorderLayout());
        statusBar.setBorder(BorderFactory.createEtchedBorder());
        statusBar.add(statusLabel, BorderLayout.WEST);
        statusBar.add(userLabel, BorderLayout.EAST);
        add(statusBar, BorderLayout.SOUTH);
    }

    private void setupMenuBar() {
        JMenuBar menuBar = new JMenuBar();

        JMenu fileMenu = new JMenu("文件");
        JMenuItem loginItem = new JMenuItem("登录/注册");
        JMenuItem logoutItem = new JMenuItem("登出");
        JMenuItem exitItem = new JMenuItem("退出");
        fileMenu.add(loginItem);
        fileMenu.add(logoutItem);
        fileMenu.addSeparator();
        fileMenu.add(exitItem);

        JMenu helpMenu = new JMenu("帮助");
        JMenuItem aboutItem = new JMenuItem("关于");
        helpMenu.add(aboutItem);

        menuBar.add(fileMenu);
        menuBar.add(helpMenu);
        setJMenuBar(menuBar);

        loginItem.addActionListener(e -> showLoginDialog());
        logoutItem.addActionListener(e -> performLogout());
        exitItem.addActionListener(e -> System.exit(0));
        aboutItem.addActionListener(e -> showAboutDialog());
    }

    private void showLoginDialog() {
        LoginDialog dialog = new LoginDialog(this);
        dialog.setVisible(true);
        updateUserInfo();
    }

    private void performLogout() {
        try {
            WubiApiClient.logout();
            statusLabel.setText("状态: 已登出");
        } catch (Exception e) {
            statusLabel.setText("登出失败: " + e.getMessage());
        }
        updateUserInfo();
    }

    private void updateUserInfo() {
        if (WubiApiClient.isAuth()) {
            userLabel.setText("已登录");
        } else {
            userLabel.setText("未登录");
        }
    }

    private void showAboutDialog() {
        JOptionPane.showMessageDialog(this,
                "五笔打字练习系统\n\n" +
                "Java Swing 桌面客户端\n" +
                "连接后端: http://localhost:3000\n" +
                "版本: 1.0.0",
                "关于",
                JOptionPane.INFORMATION_MESSAGE);
    }
}
