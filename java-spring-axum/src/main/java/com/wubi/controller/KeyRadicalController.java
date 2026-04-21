package com.wubi.controller;

import com.wubi.entity.KeyRadical;
import com.wubi.service.KeyRadicalService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;
import java.util.List;

@RestController
@RequestMapping("/api")
public class KeyRadicalController {
    @Autowired
    private KeyRadicalService keyRadicalService;

    @GetMapping("/key-radicals")
    public ResponseEntity<List<KeyRadical>> getAllKeyRadicals() {
        return ResponseEntity.ok(keyRadicalService.getAllKeyRadicals());
    }

    @GetMapping("/key-radicals/{key}")
    public ResponseEntity<KeyRadical> getKeyRadicalByKey(@PathVariable String key) {
        return keyRadicalService.getKeyRadicalByKey(key)
                .map(ResponseEntity::ok)
                .orElse(ResponseEntity.notFound().build());
    }
}
