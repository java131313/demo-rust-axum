package com.wubi.dto;

import jakarta.validation.constraints.NotBlank;

public class UpdateWubiCodeRequest {
    @NotBlank
    private String character;

    @NotBlank
    private String code;

    public String getCharacter() {
        return character;
    }

    public void setCharacter(String character) {
        this.character = character;
    }

    public String getCode() {
        return code;
    }

    public void setCode(String code) {
        this.code = code;
    }
}
