package com.wubi.client.ui;

import com.wubi.client.api.WubiApiClient;
import com.wubi.client.model.User;

import javax.swing.*;
import java.awt.*;
import java.awt.event.KeyAdapter;
import java.awt.event.KeyEvent;

public class LoginDialog extends JDialog {

    private JTextField usernameField;
    private JPasswordField passwordField;
    private JButton loginButton;
    private JButton registerButton;
    private JLabel statusLabel;

    public LoginDialog(JFrame parent) {
        super(parent, "五笔打字练习 - 登录", true);
        initComponents();
        setLocationRelativeTo(parent);
    }

    private void initComponents() {
        setDefaultCloseOperation(DISPOSE_ON_CLOSE);
        setSize(400, 350);
        setResizable(false);

        JPanel mainPanel = new JPanel();
        mainPanel.setLayout(new BoxLayout(mainPanel, BoxLayout.Y_AXIS));
        mainPanel.setBorder(BorderFactory.createEmptyBorder(20, 30, 20, 30));

        JLabel titleLabel = new JLabel("五笔打字练习系统");
        titleLabel.setFont(new Font("SansSerif", Font.BOLD, 24));
        titleLabel.setAlignmentX(Component.CENTER_ALIGNMENT);
        mainPanel.add(titleLabel);
        mainPanel.add(Box.createVerticalStrut(20));

        // Username
        JPanel usernamePanel = new JPanel(new BorderLayout(5, 5));
        usernamePanel.add(new JLabel("用户名:"), BorderLayout.WEST);
        usernameField = new JTextField();
        usernamePanel.add(usernameField, BorderLayout.CENTER);
        usernamePanel.setAlignmentX(Component.CENTER_ALIGNMENT);
        mainPanel.add(usernamePanel);
        mainPanel.add(Box.createVerticalStrut(10));

        // Password
        JPanel passwordPanel = new JPanel(new BorderLayout(5, 5));
        passwordPanel.add(new JLabel("密码:"), BorderLayout.WEST);
        passwordField = new JPasswordField();
        passwordField.addKeyListener(new KeyAdapter() {
            @Override
            public void keyPressed(KeyEvent e) {
                if (e.getKeyCode() == KeyEvent.VK_ENTER) {
                    performLogin();
                }
            }
        });
        passwordPanel.add(passwordField, BorderLayout.CENTER);
        passwordPanel.setAlignmentX(Component.CENTER_ALIGNMENT);
        mainPanel.add(passwordPanel);
        mainPanel.add(Box.createVerticalStrut(20));

        // Buttons
        JPanel buttonPanel = new JPanel(new FlowLayout());
        loginButton = new JButton("登录");
        registerButton = new JButton("注册");
        buttonPanel.add(loginButton);
        buttonPanel.add(registerButton);
        buttonPanel.setAlignmentX(Component.CENTER_ALIGNMENT);
        mainPanel.add(buttonPanel);
        mainPanel.add(Box.createVerticalStrut(10));

        // Status
        statusLabel = new JLabel(" ");
        statusLabel.setForeground(Color.RED);
        statusLabel.setAlignmentX(Component.CENTER_ALIGNMENT);
        mainPanel.add(statusLabel);

        // Action listeners
        loginButton.addActionListener(e -> performLogin());
        registerButton.addActionListener(e -> performRegister());

        getContentPane().add(mainPanel);
    }

    private void performLogin() {
        String username = usernameField.getText().trim();
        String password = new String(passwordField.getPassword());

        if (username.isEmpty() || password.isEmpty()) {
            statusLabel.setText("请输入用户名和密码");
            return;
        }

        loginButton.setEnabled(false);
        statusLabel.setText("登录中...");
        statusLabel.setForeground(Color.BLUE);

        SwingWorker<String, Void> worker = new SwingWorker<>() {
            @Override
            protected String doInBackground() {
                try {
                    return WubiApiClient.login(username, password);
                } catch (Exception ex) {
                    return "ERROR:" + ex.getMessage();
                }
            }

            @Override
            protected void done() {
                loginButton.setEnabled(true);
                try {
                    String result = get();
                    if (result.startsWith("ERROR:")) {
                        statusLabel.setText("登录失败: " + result.substring(6));
                        statusLabel.setForeground(Color.RED);
                    } else {
                        WubiApiClient.setAuthToken(result);
                        statusLabel.setText("登录成功!");
                        statusLabel.setForeground(new Color(0, 150, 0));
                        SwingUtilities.invokeLater(() -> {
                            LoginDialog.this.dispose();
                        });
                    }
                } catch (Exception ex) {
                    statusLabel.setText("登录失败: " + ex.getMessage());
                    statusLabel.setForeground(Color.RED);
                }
            }
        };
        worker.execute();
    }

    private void performRegister() {
        String username = usernameField.getText().trim();
        String password = new String(passwordField.getPassword());

        if (username.isEmpty() || password.isEmpty()) {
            statusLabel.setText("请输入用户名和密码");
            return;
        }

        String email = JOptionPane.showInputDialog(this, "请输入邮箱:", "注册", JOptionPane.QUESTION_MESSAGE);
        if (email == null || email.trim().isEmpty()) {
            return;
        }

        registerButton.setEnabled(false);
        statusLabel.setText("注册中...");
        statusLabel.setForeground(Color.BLUE);

        SwingWorker<User, Void> worker = new SwingWorker<>() {
            @Override
            protected User doInBackground() {
                try {
                    return WubiApiClient.register(username, email, password);
                } catch (Exception ex) {
                    throw new RuntimeException(ex);
                }
            }

            @Override
            protected void done() {
                registerButton.setEnabled(true);
                try {
                    User user = get();
                    JOptionPane.showMessageDialog(LoginDialog.this,
                            "注册成功! 用户: " + user.getUsername(),
                            "注册成功",
                            JOptionPane.INFORMATION_MESSAGE);
                    statusLabel.setText("注册成功,请登录");
                    statusLabel.setForeground(new Color(0, 150, 0));
                } catch (Exception ex) {
                    statusLabel.setText("注册失败: " + ex.getMessage());
                    statusLabel.setForeground(Color.RED);
                }
            }
        };
        worker.execute();
    }
}
