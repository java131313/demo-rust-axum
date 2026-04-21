package com.wubi.entity;

import jakarta.persistence.*;

@Entity
@Table(name = "key_radicals")
public class KeyRadical {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(name = "key_char", unique = true, nullable = false, length = 4)
    private String keyChar;

    @Column(columnDefinition = "TEXT", nullable = false)
    private String radicals;

    @Column(columnDefinition = "TEXT")
    private String formula;

    @Column(columnDefinition = "TEXT")
    private String description;

    public Long getId() {
        return id;
    }

    public void setId(Long id) {
        this.id = id;
    }

    public String getKeyChar() {
        return keyChar;
    }

    public void setKeyChar(String keyChar) {
        this.keyChar = keyChar;
    }

    public String getRadicals() {
        return radicals;
    }

    public void setRadicals(String radicals) {
        this.radicals = radicals;
    }

    public String getFormula() {
        return formula;
    }

    public void setFormula(String formula) {
        this.formula = formula;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }
}
