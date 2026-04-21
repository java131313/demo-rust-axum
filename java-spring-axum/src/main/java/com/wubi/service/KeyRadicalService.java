package com.wubi.service;

import com.wubi.entity.KeyRadical;
import com.wubi.repository.KeyRadicalRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import java.util.List;
import java.util.Optional;

@Service
public class KeyRadicalService {
    @Autowired
    private KeyRadicalRepository keyRadicalRepository;

    public List<KeyRadical> getAllKeyRadicals() {
        return keyRadicalRepository.findAll();
    }

    public Optional<KeyRadical> getKeyRadicalByKey(String keyChar) {
        return keyRadicalRepository.findByKeyChar(keyChar);
    }
}
