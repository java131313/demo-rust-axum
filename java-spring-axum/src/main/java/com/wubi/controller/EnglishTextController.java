package com.wubi.controller;

import com.wubi.entity.EnglishText;
import com.wubi.service.EnglishTextService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;
import java.util.List;

@RestController
@RequestMapping("/api")
public class EnglishTextController {
    @Autowired
    private EnglishTextService englishTextService;

    @GetMapping("/english-texts")
    public ResponseEntity<List<EnglishText>> getAllEnglishTexts() {
        return ResponseEntity.ok(englishTextService.getAllEnglishTexts());
    }
}
