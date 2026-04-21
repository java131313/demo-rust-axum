from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    port: int = 3000
    db_host: str = "127.0.0.1"
    db_port: int = 3306
    db_username: str = "root"
    db_password: str = "sdsSDG123*^DD"
    db_name: str = "wubi"
    jwt_secret: str = "your-secret-key-change-in-production"

    @property
    def database_url(self) -> str:
        return f"mysql+pymysql://{self.db_username}:{self.db_password}@{self.db_host}:{self.db_port}/{self.db_name}"

    class Config:
        env_file = ".env"


_settings = None


def get_settings() -> Settings:
    global _settings
    if _settings is None:
        _settings = Settings()
    return _settings
