package com.wubi.client.api;

import com.wubi.client.model.KeyRadical;
import com.wubi.client.model.User;
import com.wubi.client.model.WubiCharacter;
import com.wubi.client.model.Lesson;

import java.io.*;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

public class WubiApiClient {

    public static final String BASE_URL = "http://localhost:3000";
    private static String authToken = null;

    public static void setAuthToken(String token) {
        authToken = token;
    }

    public static String getAuthToken() {
        return authToken;
    }

    public static boolean isAuth() {
        return authToken != null && !authToken.isEmpty();
    }

    // ==================== Auth API ====================

    public static String login(String username, String password) throws IOException {
        String jsonInput = "{\"username\":\"" + escapeJson(username) + "\",\"password\":\"" + escapeJson(password) + "\"}";
        String response = sendPost("/api/login", jsonInput);
        String token = extractJsonString(response, "access_token");
        if (token != null) {
            return token;
        }
        throw new RuntimeException("Login failed: " + response);
    }

    public static User register(String username, String email, String password) throws IOException {
        String jsonInput = "{\"username\":\"" + escapeJson(username) + "\",\"email\":\"" + escapeJson(email) + "\",\"password\":\"" + escapeJson(password) + "\"}";
        String response = sendPost("/api/register", jsonInput);
        return parseUser(response);
    }

    public static void logout() throws IOException {
        sendPost("/api/logout", "{}");
        authToken = null;
    }

    // ==================== Wubi API ====================

    public static WubiCharacter getWubiCharacter(String character) throws IOException {
        String response = sendGet("/api/wubi/" + character);
        return parseWubiCharacter(response);
    }

    public static List<WubiCharacter> searchWubiRoot(String character) throws IOException {
        String response = sendGet("/api/search-wubi-root/" + character);
        return parseWubiCharacterList(response);
    }

    public static List<KeyRadical> getKeyRadicals() throws IOException {
        String response = sendGet("/api/key-radicals");
        return parseKeyRadicalList(response);
    }

    public static KeyRadical getKeyRadicalByChar(String keyChar) throws IOException {
        String response = sendGet("/api/key-radicals/" + keyChar);
        return parseKeyRadical(response);
    }

    // ==================== Lesson API ====================

    public static List<Lesson> getLessons() throws IOException {
        String response = sendGet("/api/lessons");
        return parseLessonList(response);
    }

    public static Lesson createLesson(String title, String content) throws IOException {
        String jsonInput = "{\"title\":\"" + escapeJson(title) + "\",\"content\":\"" + escapeJson(content) + "\"}";
        String response = sendPost("/api/lessons", jsonInput);
        return parseLesson(response);
    }

    public static Lesson getLesson(int id) throws IOException {
        String response = sendGet("/api/lessons/" + id);
        return parseLesson(response);
    }

    // ==================== English Texts API ====================

    public static String getEnglishTexts() throws IOException {
        return sendGet("/api/english-texts");
    }

    // ==================== Health Check ====================

    public static boolean isHealthy() {
        try {
            String response = sendGet("/api/health");
            return response != null && !response.isEmpty();
        } catch (Exception e) {
            return false;
        }
    }

    // ==================== HTTP Helpers ====================

    public static String sendGet(String path) throws IOException {
        URL url = new URL(BASE_URL + path);
        HttpURLConnection conn = (HttpURLConnection) url.openConnection();
        conn.setRequestMethod("GET");
        conn.setRequestProperty("Accept", "application/json");
        if (authToken != null) {
            conn.setRequestProperty("Authorization", "Bearer " + authToken);
        }

        int status = conn.getResponseCode();
        if (status == HttpURLConnection.HTTP_OK) {
            return readInputStream(conn.getInputStream());
        } else {
            String err = readInputStream(conn.getErrorStream());
            throw new IOException("HTTP " + status + ": " + err);
        }
    }

    private static String sendPost(String path, String jsonInput) throws IOException {
        URL url = new URL(BASE_URL + path);
        HttpURLConnection conn = (HttpURLConnection) url.openConnection();
        conn.setRequestMethod("POST");
        conn.setRequestProperty("Content-Type", "application/json");
        conn.setRequestProperty("Accept", "application/json");
        if (authToken != null) {
            conn.setRequestProperty("Authorization", "Bearer " + authToken);
        }
        conn.setDoOutput(true);

        byte[] out = jsonInput.getBytes(StandardCharsets.UTF_8);
        conn.setFixedLengthStreamingMode(out.length);
        try (OutputStream os = conn.getOutputStream()) {
            os.write(out);
        }

        int status = conn.getResponseCode();
        InputStream is = (status >= 200 && status < 300) ? conn.getInputStream() : conn.getErrorStream();
        return readInputStream(is);
    }

    private static String readInputStream(InputStream is) throws IOException {
        if (is == null) return "";
        StringBuilder sb = new StringBuilder();
        try (BufferedReader reader = new BufferedReader(new InputStreamReader(is, StandardCharsets.UTF_8))) {
            String line;
            while ((line = reader.readLine()) != null) {
                sb.append(line);
            }
        }
        return sb.toString();
    }

    // ==================== Simple JSON Parsing ====================

    private static String escapeJson(String s) {
        return s.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t");
    }

    public static String extractJsonString(String json, String key) {
        return extractJsonString(json, key, null);
    }

    public static String extractJsonString(String json, String key, String defaultVal) {
        String search = "\"" + key + "\"";
        int idx = json.indexOf(search);
        if (idx == -1) return defaultVal;
        idx = json.indexOf(':', idx) + 1;
        while (idx < json.length() && json.charAt(idx) <= ' ') idx++;
        if (idx >= json.length()) return defaultVal;
        if (json.charAt(idx) == '"') {
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
            return json.substring(start, end).replace("\\\"", "\"").replace("\\\\", "\\").replace("\\n", "\n").replace("\\t", "\t");
        }
        int end = idx;
        while (end < json.length() && json.charAt(end) != ',' && json.charAt(end) != '}') end++;
        String val = json.substring(idx, end).trim();
        return val.isEmpty() ? defaultVal : val;
    }

    private static int extractJsonInt(String json, String key, int defaultVal) {
        String val = extractJsonString(json, key);
        if (val == null) return defaultVal;
        try {
            return Integer.parseInt(val);
        } catch (NumberFormatException e) {
            return defaultVal;
        }
    }

    private static User parseUser(String json) {
        User u = new User();
        u.setId(extractJsonInt(json, "id", 0));
        u.setUsername(extractJsonString(json, "username", ""));
        u.setEmail(extractJsonString(json, "email", ""));
        u.setCreatedAt(extractJsonString(json, "created_at", ""));
        return u;
    }

    private static WubiCharacter parseWubiCharacter(String json) {
        WubiCharacter wc = new WubiCharacter();
        wc.setId(extractJsonInt(json, "id", 0));
        wc.setCharacter(extractJsonString(json, "character", ""));
        wc.setWubi86(extractJsonString(json, "wubi86", ""));
        wc.setPinyin(extractJsonString(json, "pinyin", ""));
        wc.setSimplePinyin(extractJsonString(json, "simple_pinyin", ""));
        wc.setRadicals(extractJsonString(json, "radicals", ""));
        wc.setDescription(extractJsonString(json, "description", ""));
        return wc;
    }

    private static List<WubiCharacter> parseWubiCharacterList(String json) {
        List<WubiCharacter> list = new ArrayList<>();
        int idx = 0;
        while ((idx = json.indexOf('{', idx)) != -1) {
            int end = findMatchingBrace(json, idx);
            if (end == -1) break;
            String item = json.substring(idx, end + 1);
            list.add(parseWubiCharacter(item));
            idx = end + 1;
        }
        return list;
    }

    private static KeyRadical parseKeyRadical(String json) {
        KeyRadical kr = new KeyRadical();
        kr.setId(extractJsonInt(json, "id", 0));
        kr.setKeyChar(extractJsonString(json, "key_char", ""));
        kr.setRadicals(extractJsonString(json, "radicals", ""));
        kr.setFormula(extractJsonString(json, "formula", ""));
        kr.setDescription(extractJsonString(json, "description", ""));

        String wubiChars = extractJsonString(json, "wubi_characters");
        if (wubiChars != null) {
            List<String> chars = new ArrayList<>();
            int qi = 0;
            while ((qi = wubiChars.indexOf('"', qi)) != -1) {
                qi++;
                int qe = wubiChars.indexOf('"', qi);
                if (qe == -1) break;
                chars.add(wubiChars.substring(qi, qe));
                qi = qe + 1;
            }
            kr.setWubiCharacters(chars);
        }
        return kr;
    }

    private static List<KeyRadical> parseKeyRadicalList(String json) {
        List<KeyRadical> list = new ArrayList<>();
        int idx = 0;
        while ((idx = json.indexOf('{', idx)) != -1) {
            int end = findMatchingBrace(json, idx);
            if (end == -1) break;
            String item = json.substring(idx, end + 1);
            list.add(parseKeyRadical(item));
            idx = end + 1;
        }
        return list;
    }

    private static Lesson parseLesson(String json) {
        Lesson l = new Lesson();
        l.setId(extractJsonInt(json, "id", 0));
        l.setTitle(extractJsonString(json, "title", ""));
        l.setContent(extractJsonString(json, "content", ""));
        l.setCreatedAt(extractJsonString(json, "created_at", ""));
        return l;
    }

    private static List<Lesson> parseLessonList(String json) {
        List<Lesson> list = new ArrayList<>();
        int idx = 0;
        while ((idx = json.indexOf('{', idx)) != -1) {
            int end = findMatchingBrace(json, idx);
            if (end == -1) break;
            String item = json.substring(idx, end + 1);
            list.add(parseLesson(item));
            idx = end + 1;
        }
        return list;
    }

    private static int findMatchingBrace(String json, int start) {
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
}
