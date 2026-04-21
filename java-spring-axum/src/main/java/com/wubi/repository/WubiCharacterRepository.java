package com.wubi.repository;

import com.wubi.entity.WubiCharacter;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.stereotype.Repository;
import java.util.Optional;

@Repository
public interface WubiCharacterRepository extends JpaRepository<WubiCharacter, Long> {
    Optional<WubiCharacter> findByCharacter(String character);
}
