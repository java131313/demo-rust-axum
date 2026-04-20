package com.wubi.client.model;

public class WubiCharacter {
    private int id;
    private String character;
    private String wubi86;
    private String pinyin;
    private String simple_pinyin;
    private String radicals;
    private String description;

    public int getId() { return id; }
    public void setId(int id) { this.id = id; }
    public String getCharacter() { return character; }
    public void setCharacter(String character) { this.character = character; }
    public String getWubi86() { return wubi86; }
    public void setWubi86(String wubi86) { this.wubi86 = wubi86; }
    public String getPinyin() { return pinyin; }
    public void setPinyin(String pinyin) { this.pinyin = pinyin; }
    public String getSimplePinyin() { return simple_pinyin; }
    public void setSimplePinyin(String simple_pinyin) { this.simple_pinyin = simple_pinyin; }
    public String getRadicals() { return radicals; }
    public void setRadicals(String radicals) { this.radicals = radicals; }
    public String getDescription() { return description; }
    public void setDescription(String description) { this.description = description; }
}
