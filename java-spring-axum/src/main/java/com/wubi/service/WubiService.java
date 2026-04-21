package com.wubi.service;

import com.wubi.dto.UpdateWubiCodeRequest;
import com.wubi.entity.WubiCharacter;
import com.wubi.repository.WubiCharacterRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import java.util.Optional;

@Service
public class WubiService {
    @Autowired
    private WubiCharacterRepository wubiCharacterRepository;

    public Optional<WubiCharacter> getWubiCode(String character) {
        return wubiCharacterRepository.findByCharacter(character);
    }

    public WubiCharacter updateWubiCode(UpdateWubiCodeRequest request) {
        Optional<WubiCharacter> existing = wubiCharacterRepository.findByCharacter(request.getCharacter());

        WubiCharacter wubiChar;
        if (existing.isPresent()) {
            wubiChar = existing.get();
            wubiChar.setFullCode(request.getCode());
        } else {
            wubiChar = new WubiCharacter();
            wubiChar.setCharacter(request.getCharacter());
            wubiChar.setFullCode(request.getCode());
        }

        return wubiCharacterRepository.save(wubiChar);
    }
}
