package com.wubi.client.model;

import java.util.List;

public class KeyRadical {
    private int id;
    private String key_char;
    private String radicals;
    private String formula;
    private String description;
    private List<String> wubi_characters;

    public int getId() { return id; }
    public void setId(int id) { this.id = id; }
    public String getKeyChar() { return key_char; }
    public void setKeyChar(String key_char) { this.key_char = key_char; }
    public String getRadicals() { return radicals; }
    public void setRadicals(String radicals) { this.radicals = radicals; }
    public String getFormula() { return formula; }
    public void setFormula(String formula) { this.formula = formula; }
    public String getDescription() { return description; }
    public void setDescription(String description) { this.description = description; }
    public List<String> getWubiCharacters() { return wubi_characters; }
    public void setWubiCharacters(List<String> wubi_characters) { this.wubi_characters = wubi_characters; }
}
