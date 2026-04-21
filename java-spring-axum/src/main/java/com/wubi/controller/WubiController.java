package com.wubi.controller;

import com.wubi.dto.UpdateWubiCodeRequest;
import com.wubi.entity.WubiCharacter;
import com.wubi.service.WubiService;
import jakarta.validation.Valid;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api")
public class WubiController {
    @Autowired
    private WubiService wubiService;

    @GetMapping("/wubi/{character}")
    public ResponseEntity<?> getWubiCode(@PathVariable String character) {
        if (character.length() != 1) {
            return ResponseEntity.badRequest().body("{\"error\": \"Character must be a single character\"}");
        }

        return wubiService.getWubiCode(character)
                .map(ResponseEntity::ok)
                .orElse(ResponseEntity.notFound().build());
    }

    @PutMapping("/wubi-code")
    public ResponseEntity<WubiCharacter> updateWubiCode(@Valid @RequestBody UpdateWubiCodeRequest request) {
        return ResponseEntity.ok(wubiService.updateWubiCode(request));
    }
}
